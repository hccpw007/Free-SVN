use crate::models::error::AppError;
use log;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::task::spawn_blocking;
use tokio::time::{timeout, Duration};

/// 全局取消标志。
///
/// ## 取消机制设计说明
///
/// 全局取消采用 `AtomicBool + Mutex<Option<Child>>` 双层协作设计：
///
/// 1. **AtomicBool (CANCELLED)** — 轻量级取消信号。
///    - `run_svn()` 入口处调用 `is_cancelled()` 检查，被取消时快速返回 Cancelled 错误。
///    - `set_cancelled(true)` 由 cancel command 触发。
///    - 闭包内不轮询 AtomicBool（依赖超时机制兜底）。
///
/// 2. **Mutex<Option<Child>> (CURRENT_CHILD)** — 子进程句柄存储。
///    - 句柄的存入和取出都在 spawn_blocking 闭包内完成，生命周期不逃逸。
///    - `kill_current_process()` 通过 Mutex 获取 Child 所有权后 kill+wait。
///    - Mutex 仅保护赋值/取值的两个瞬间，不阻塞 wait_with_output。
///
/// 3. **协作流程**：设置 CANCELLED=true → `kill_current_process()` 终止子进程 →
///    超时机制返回 Timeout 或子进程被 kill 后 wait_with_output 返回 →
///    `kill_and_cleanup()` 执行 svn cleanup → 重置 CANCELLED=false。
///
/// 4. **线程安全性**：AtomicBool 使用 SeqCst 排序确保跨线程可见性。
///    Mutex 的 guard 在短暂持有后立即释放，不发生锁争用。
pub static CANCELLED: AtomicBool = AtomicBool::new(false);

/// 当前 svn 子进程句柄（Mutex 保护）。
///
/// ## 安全使用规则
///
/// ### 核心约束
/// 子进程的 spawn、Child 存取、wait 全部在 spawn_blocking 闭包内完成，生命周期不逃逸到闭包外。
/// Mutex 不保护 wait_with_output（该阶段 Guard 已释放），因此取消操作不会被锁阻塞。
///
/// ### 存取流程
/// spawn → `*guard = Some(child)`（Mutex 存入）→ guard 立即释放 →
/// `guard.take()`（重新加锁取出所有权）→ guard 立即释放 → child.wait_with_output()。
/// Guard 持有时间仅限赋值和取值的两个极短瞬间。
///
/// ### 取消竞争条件
/// - 若 `kill_current_process()` 在存入之后、取出之前被调用，kill 操作能获取到 Child。
/// - 若在取出之后（正在 wait_with_output）被调用，CURRENT_CHILD 中为 None，kill 为空操作。
/// - 无论哪种情况，进程都能被正常终止（kill 或超时机制兜底）。
pub static CURRENT_CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);

const DEFAULT_TIMEOUT_SECS: u64 = 60;
const BLAME_TIMEOUT_SECS: u64 = 120;

/// 认证提示关键词
const AUTH_KEYWORDS: &[&str] = &[
    "Authentication realm",
    "Password:",
    "Username:",
    "认证领域",
    "密码:",
    "用户名:",
];

/// 基础 SVN 参数（防止交互式认证对话框弹出）
const BASE_SVN_ARGS: &[&str] = &[
    "--non-interactive",
    "--trust-server-cert-failures",
    "unknown-ca,cn-mismatch,expired,not-yet-valid,other",
];

/// 认证错误检测关键词（v5 新增）
const AUTH_ERROR_KEYWORDS: &[&str] = &[
    "Authentication realm",
    "认证领域",
    "authorization failed",
    "认证失败",
    "access denied",
    "No credentials",
    "密码不正确",
    "Password incorrect",
    "E215004",
    "E170001",
    "E175002",
];

// ── 内置 SVN 资源路径 ─────────────────────────────

/// 全局存储 Tauri 的资源目录路径（用于定位内置 SVN 二进制）。
/// 在 lib.rs::setup() 中通过 `set_svn_resource_dir()` 注入。
static SVN_RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 设置内置 SVN 资源目录（在 app setup 时调用）。
pub fn set_svn_resource_dir(dir: PathBuf) {
    let _ = SVN_RESOURCE_DIR.set(dir);
}

/// 获取 SVN 可执行文件路径。
///
/// ## 生产模式
/// 如果 `SVN_RESOURCE_DIR` 已设置且对应平台的 SVN 二进制文件存在，
/// 返回内置路径（如 `{resource_dir}/svn/macos/svn`）。
///
/// ## 开发模式
/// 回退到系统 PATH 中的 `svn` 命令。
pub fn get_svn_path() -> PathBuf {
    if let Some(resource_dir) = SVN_RESOURCE_DIR.get() {
        #[cfg(target_os = "macos")]
        let svn_relative = PathBuf::from("svn/macos/svn");
        #[cfg(target_os = "windows")]
        let svn_relative = PathBuf::from("svn/windows/svn.exe");
        #[cfg(target_os = "linux")]
        let svn_relative = PathBuf::from("svn/linux/svn");

        let bundled = resource_dir.join(&svn_relative);
        if bundled.exists() {
            return bundled;
        }
    }
    // 开发阶段统一使用系统 svn（PATH 中的 svn 命令）
    PathBuf::from("svn")
}

/// 获取运行 SVN 进程所需的环境变量（如 Linux 的 LD_LIBRARY_PATH）。
///
/// 对于 Linux 平台，由于内置 svn 使用 musl 编译且依赖库在 `lib/` 子目录，
/// 需要设置 `LD_LIBRARY_PATH` 指向 `{svn_dir}/lib/`。
/// macOS 和 Windows 不需要额外环境变量（分别使用 @executable_path 和同目录查找）。
pub fn get_svn_env() -> Vec<(String, String)> {
    #[cfg(target_os = "linux")]
    {
        let mut env = Vec::new();
        if let Some(resource_dir) = SVN_RESOURCE_DIR.get() {
            let lib_dir = resource_dir.join("svn/linux/lib");
            if lib_dir.exists() {
                env.push(("LD_LIBRARY_PATH".to_string(), lib_dir.to_string_lossy().to_string()));
            }
        }
        return env;
    }
    #[cfg(not(target_os = "linux"))]
    Vec::new()
}

pub fn get_timeout_secs(args: &[&str]) -> u64 {
    if args.contains(&"blame") {
        BLAME_TIMEOUT_SECS
    } else {
        DEFAULT_TIMEOUT_SECS
    }
}

pub fn validate_path(path: &str) -> Result<&str, AppError> {
    if path.is_empty() {
        return Err(AppError::InvalidInput("path cannot be empty".into()));
    }
    if path.contains('\0') {
        return Err(AppError::InvalidInput("path contains null character".into()));
    }
    Ok(path)
}

// ── 取消检测 ──────────────────────────────────────────

pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}

pub fn set_cancelled(val: bool) {
    CANCELLED.store(val, Ordering::SeqCst);
}

pub fn has_child_process() -> bool {
    CURRENT_CHILD.lock().map(|guard| guard.is_some()).unwrap_or(false)
}

// ── 认证检测 ──────────────────────────────────────────

fn check_auth_hint(stderr: &str) -> bool {
    AUTH_KEYWORDS.iter().any(|kw| stderr.contains(kw))
}

// ── 网络检测 ──────────────────────────────────────────

fn extract_host(url: &str) -> Result<String, AppError> {
    let without_scheme = url
        .strip_prefix("svn://")
        .or_else(|| url.strip_prefix("svn+ssh://"))
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| AppError::InvalidInput(format!("unrecognized URL format: {}", url)))?;

    let host = without_scheme
        .split('/')
        .next()
        .ok_or_else(|| AppError::InvalidInput(format!("host not found in URL: {}", url)))?;

    Ok(host.to_string())
}

/// 检测网络是否可达（5s 超时 TcpStream 连接尝试）
pub async fn check_network(server_url: &str) -> Result<(), AppError> {
    let host = extract_host(server_url)?;

    let ports: &[u16] = if server_url.starts_with("https://") {
        &[443]
    } else if server_url.starts_with("svn+ssh://") {
        &[22]
    } else {
        &[3690, 443]
    };

    for &port in ports {
        let addr = format!("{}:{}", host, port);
        match timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => return Ok(()),
            _ => continue,
        }
    }

    log::warn!("check_network 失败: host={}", host);
    Err(AppError::NetworkUnreachable)
}

// ── 认证检测（v5 新增） ─────────────────────────────

/// 判断 stderr 是否为认证错误
fn is_auth_error(stderr: &str) -> bool {
    AUTH_ERROR_KEYWORDS.iter().any(|kw| stderr.contains(kw))
}

/// 提取 realm 信息（用于前端显示）
/// 从 SVN 错误输出中提取认证领域名（仅用于 AuthDialog 的上下文提示）。
/// 无论是否找到 realm，都不影响错误消息的完整性。
fn extract_realm(stderr: &str) -> String {
    for line in stderr.lines() {
        if line.contains("Authentication realm") || line.contains("认证领域") {
            return line.trim().to_string();
        }
    }
    String::new()  // 无 realm 行时返回空串，不参与错误消息
}

// ── 核心执行（v5 增加 credentials 支持） ────────────

/// 在 spawn_blocking 闭包内同步执行 SVN 命令。
/// 子进程的 spawn、Child 存取、wait 全部在闭包内完成，生命周期不逃逸到闭包外。
/// 取消信号通过 AtomicBool 检测：闭包内每步操作前检查 CANCELLED 标志。
fn run_svn_sync(
    svn_path: &PathBuf,
    args: &[String],
    cwd: &str,
    credentials: Option<&crate::svn::types::SvnCredentials>,
) -> Result<String, AppError> {
    use std::io::Write;
    let start = Instant::now();

    // 日志记录（仅掩码密码参数）
    // 注意：--password 命令行传参已被业务设计 §2.2 禁止，仅保留 --password-from-stdin 掩码
    let log_args = if credentials.is_some() {
        let masked: Vec<String> = args.iter().map(|a| {
            if a.starts_with("--password-from-stdin") {
                "--password-from-stdin [***]".to_string()
            } else {
                a.clone()
            }
        }).collect();
        masked.join(" ")
    } else {
        args.join(" ")
    };
    log::info!("svn {} (cwd: {})", log_args, cwd);

    let mut cmd = Command::new(svn_path);
    cmd.args(args).current_dir(cwd).envs(get_svn_env());

    let child = if let Some(creds) = credentials {
        // 有凭据：通过 stdin 传递密码
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut c = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::SvnNotFound
            } else {
                AppError::Io(e)
            }
        })?;
        // stdin 写入密码
        if let Some(mut stdin) = c.stdin.take() {
            stdin.write_all(creds.password.as_bytes()).map_err(AppError::Io)?;
            stdin.flush().map_err(AppError::Io)?;
        }
        c
    } else {
        // 无凭据：正常执行
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::SvnNotFound
            } else {
                AppError::Io(e)
            }
        })?
    };

    // v5 修复：子进程句柄的存/取/等待全部在 spawn_blocking 闭包内完成。
    // 将 child 存入 CURRENT_CHILD 供 kill_current_process() 取消操作访问
    // Guard 仅在存入与取出之间持有，粒度为两次 lock/unlock 之间
    if let Ok(mut guard) = CURRENT_CHILD.lock() {
        *guard = Some(child);
    }
    // 取出所有权等待完成（取消操作可能已在间隙中 kill 了进程）
    let child = CURRENT_CHILD.lock().ok().and_then(|mut g| g.take())
        .ok_or_else(|| AppError::Repo("run_svn_sync: 子进程句柄丢失".into()))?;
    let output = child.wait_with_output().map_err(AppError::Io)?;
    let elapsed = start.elapsed();
    log::info!("svn 完成 (耗时: {:?}, 退出码: {:?})", elapsed, output.status.code());

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("svn stderr: {}", stderr);

        // 检测是否为认证错误 — 无论是否找到 realm，错误消息始终使用完整 stderr
        if is_auth_error(&stderr) {
            let _realm = extract_realm(&stderr);  // realm 仅用于内部上下文（暂未使用）
            return Err(AppError::SvnAuthFailed(stderr.to_string()));
        }

        Err(AppError::SvnCommand(stderr.to_string()))
    }
}

/// 异步执行 SVN 命令（带超时控制 + 取消检测 + 认证检测 + 凭据支持）
/// credentials: None = --non-interactive 模式（走系统缓存），Some = 使用输入凭据
pub async fn run_svn(
    args: &[&str],
    cwd: &str,
    credentials: Option<&crate::svn::types::SvnCredentials>,
) -> Result<String, AppError> {
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    let svn_path = get_svn_path();
    let timeout_secs = get_timeout_secs(args);

    // 构造参数：有凭据时移除 --non-interactive 并追加 --username
    let mut all_args: Vec<String> = Vec::new();

    if let Some(creds) = credentials {
        // 有凭据：根据 save_to_cache 决定是否保留 --non-interactive
        // --non-interactive：SVN 不会将凭据写入 ~/.subversion/auth/ 缓存
        // 移除 --non-interactive：SVN 会自动缓存凭据
        for base_arg in BASE_SVN_ARGS {
            if creds.save_to_cache && *base_arg == "--non-interactive" {
                continue; // 用户勾选了保存密码：移除 --non-interactive，让 svn 缓存凭据
            }
            all_args.push(base_arg.to_string());
        }
        all_args.push("--username".to_string());
        all_args.push(creds.username.clone());
        // --password-from-stdin：让 SVN 从 stdin 管道读取密码而非交互式提示
        all_args.push("--password-from-stdin".to_string());
    } else {
        // 无凭据：保持 --non-interactive
        all_args.extend(BASE_SVN_ARGS.iter().map(|s| s.to_string()));
    }

    // 追加操作特定参数
    // 有凭据时 --password-from-stdin 已在上面添加，run_svn_sync
    // 通过 stdin 管道写入密码
    for arg in args.iter() {
        if credentials.map_or(false, |c| c.save_to_cache) && *arg == "--non-interactive" {
            continue; // save_to_cache=true 时移除 args 中的 --non-interactive 避免重复
        }
        all_args.push(arg.to_string());
    }

    let cwd = cwd.to_string();
    // v5 修复：克隆 credentials 后传入 spawn_blocking（引用无法跨 'static 边界）
    let creds = credentials.cloned();

    let result = timeout(
        Duration::from_secs(timeout_secs),
        spawn_blocking(move || {
            run_svn_sync(&svn_path, &all_args, &cwd, creds.as_ref())
        }),
    )
    .await;

    match result {
        Ok(inner) => {
            // inner: Result<Result<String, AppError>, JoinError> —
            // 外层 Result 来自 JoinHandle，内层来自 run_svn_sync
            inner.map_err(|e| AppError::Repo(format!("spawn_blocking error: {}", e)))?
        }
        Err(_elapsed) => {
            kill_current_process();
            log::warn!("SVN 命令超时 ({}s)", timeout_secs);
            Err(AppError::Timeout(format!("SVN command timed out after {}s", timeout_secs)))
        }
    }
}

// ── 进程管理 ──────────────────────────────────────────

/// 带进度推送的 SVN 命令执行。
///
/// 与 `run_svn` 不同，此函数通过双管道逐行读取 stdout/stderr，
/// 实时解析进度并通过 Tauri event 推送到前端。
/// 使用场景：checkout / update / commit / switch / merge / export
pub async fn run_svn_with_progress(
    args: &[&str],
    cwd: &str,
    credentials: Option<&crate::svn::types::SvnCredentials>,
    app_handle: tauri::AppHandle,
    operation: &str,
) -> Result<String, AppError> {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::mpsc;
    use std::thread;
    use tauri::Emitter;

    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    // 1. emit operation:started
    app_handle.emit("operation:started", serde_json::json!({
        "operation": operation
    })).ok();

    let svn_path = get_svn_path();
    let svn_path_str = svn_path.to_string_lossy().to_string();

    // 构建 args
    let mut all_args: Vec<String> = Vec::new();
    all_args.extend(BASE_SVN_ARGS.iter().map(|s| s.to_string()));
    for arg in args.iter() {
        all_args.push(arg.to_string());
    }

    let cwd_str = cwd.to_string();
    let creds = credentials.cloned();
    let operation_owned = operation.to_string();
    let ah = app_handle.clone();

    let result = spawn_blocking(move || {
        let start = Instant::now();

        // 日志
        let log_args = all_args.join(" ");
        log::info!("svn_with_progress {} (cwd: {})", log_args, cwd_str);

        // spawn 子进程
        let mut cmd = Command::new(&svn_path_str);
        cmd.args(&all_args)
            .current_dir(&cwd_str)
            .envs(get_svn_env())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::NotFound {
                    Err(AppError::SvnNotFound)
                } else {
                    Err(AppError::Io(e))
                };
            }
        };

        if let Some(c) = creds.as_ref() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(c.password.as_bytes());
                let _ = stdin.flush();
            }
        }

        // 先获取 stdout/stderr 句柄（必须在存储 child 之前）
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        // 然后在全局存储子进程句柄
        if let Ok(mut guard) = CURRENT_CHILD.lock() {
            *guard = Some(child);
        }

        // channel 通信
        let (tx_stdout, rx_stdout) = mpsc::channel::<String>();
        let (tx_stderr, rx_stderr) = mpsc::channel::<String>();
        let (tx_panic, rx_panic) = mpsc::channel::<String>();

        // stdout 线程
        let tx_s = tx_stdout.clone();
        let tx_p = tx_panic.clone();
        let op_s = operation_owned.clone();
        let stdout_thread = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(handle) = stdout_handle {
                    let reader = BufReader::new(handle);
                    for line in reader.lines() {
                        if is_cancelled() {
                            break;
                        }
                        match line {
                            Ok(text) => {
                                let _ = tx_s.send(text);
                            }
                            Err(_) => {
                                // pipe closed — send marker
                                let _ = tx_s.send(format!("__STDOUT_CLOSED__:{}", op_s));
                                break;
                            }
                        }
                    }
                }
            }));
            if result.is_err() {
                let _ = tx_p.send(format!("stdout_thread_panic:{}", op_s));
            }
        });

        // stderr 线程
        let tx_e = tx_stderr.clone();
        let tx_p2 = tx_panic.clone();
        let op_e = operation_owned.clone();
        let stderr_thread = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(handle) = stderr_handle {
                    let reader = BufReader::new(handle);
                    for line in reader.lines() {
                        if is_cancelled() {
                            break;
                        }
                        if let Ok(text) = line {
                            let _ = tx_e.send(text);
                        }
                    }
                }
            }));
            if result.is_err() {
                let _ = tx_p2.send(format!("stderr_thread_panic:{}", op_e));
            }
        });

        // 释放 sender（主循环不再持有 sender，receiver 在双方都关闭时断开）
        drop(tx_stdout);
        drop(tx_stderr);

        // ── 主循环：接收并处理行 ──
        let mut stdout_done = false;
        let mut stderr_done = false;
        let mut last_progress_time = Instant::now();
        let mut file_count: u32 = 0;
        let mut completed_count: u32 = 0;
        let mut combined_stdout = String::new();

        // ── 主循环开始前声明文件行判定函数 ──
        let is_file_line = |line: &str| -> Option<String> {
            let op = operation_owned.as_str();
            if op == "checkout" || op == "export" {
                let trimmed = line.trim_start();
                if trimmed.starts_with('A') || trimmed.starts_with('U') {
                    let rest = trimmed[1..].trim_start();
                    if !rest.is_empty() { return Some(rest.to_string()); }
                }
            } else if op == "commit" {
                if line.trim_start().starts_with("Sending") {
                    if let Some(path) = line.trim_start().strip_prefix("Sending") {
                        let rest = path.trim_start();
                        if !rest.is_empty() { return Some(rest.to_string()); }
                    }
                }
            } else {
                let trimmed = line.trim_start();
                if !trimmed.is_empty() {
                    let c = trimmed.as_bytes()[0];
                    if c == b'A' || c == b'U' || c == b'D' || c == b'C' {
                        let rest = trimmed[1..].trim_start();
                        if !rest.is_empty() { return Some(rest.to_string()); }
                    }
                }
            }
            None
        };

        // 检测 stderr 进度百分比
        let extract_percentage = |line: &str| -> Option<u8> {
            let trimmed = line.trim_start();
            if trimmed.starts_with("...") {
                let after_dots = trimmed.trim_start_matches("...").trim_start();
                if let Some(pct_str) = after_dots.strip_suffix('%') {
                    return pct_str.trim().parse::<u8>().ok();
                }
            }
            if trimmed.contains('%') {
                if let Some(pct_end) = trimmed.find('%') {
                    let pct_part = &trimmed[..pct_end];
                    let num_part = pct_part.trim();
                    if num_part.chars().all(|c| c.is_ascii_digit()) {
                        return num_part.parse::<u8>().ok();
                    }
                }
            }
            None
        };

        loop {
            // 检查取消
            if is_cancelled() {
                break;
            }

            // 检查 panic 信号
            if let Ok(panic_msg) = rx_panic.try_recv() {
                log::warn!("线程 panic: {}", panic_msg);
                break;
            }

            // 接收 stdout 行
            if !stdout_done {
                match rx_stdout.try_recv() {
                    Ok(line) => {
                        combined_stdout.push_str(&line);
                        combined_stdout.push('\n');

                        // 检测 stdout 关闭标记
                        if line.starts_with("__STDOUT_CLOSED__:") {
                            ah.emit("operation:line", serde_json::json!({
                                "operation": operation_owned,
                                "filePath": "",
                                "status": "completed",
                                "isMarker": true,
                            })).ok();
                            continue;
                        }

                        // 解析文件行
                        if let Some(file_path) = is_file_line(&line) {
                            file_count += 1;
                            completed_count += 1;
                            ah.emit("operation:line", serde_json::json!({
                                "operation": operation_owned,
                                "filePath": file_path,
                                "status": "completed"
                            })).ok();
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => { /* no data */ }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        stdout_done = true;
                    }
                }
            }

            // 接收 stderr 行
            if !stderr_done {
                match rx_stderr.try_recv() {
                    Ok(line) => {
                        // 解析百分比
                        let percent = extract_percentage(&line);

                        if let Some(pct) = percent {
                            let now = Instant::now();
                            // throttle: 首条立即发送 + 200ms 窗口
                            if last_progress_time.elapsed() >= Duration::from_millis(200) {
                                last_progress_time = now;
                                ah.emit("operation:progress", serde_json::json!({
                                    "operation": operation_owned,
                                    "percent": pct,
                                    "stage": "processing",
                                    "fileCount": file_count,
                                    "completedCount": completed_count,
                                    "pendingCount": 0u32,
                                    "speed": "",
                                    "elapsed": "",
                                })).ok();
                            }
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        stderr_done = true;
                    }
                }
            }

            // 双方都断开则退出
            if stdout_done && stderr_done {
                break;
            }

            // 避免忙等
            thread::sleep(Duration::from_millis(10));
        }

        // 等待线程完成（超时兜底）
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();

        // 取回子进程句柄并 wait
        let child = CURRENT_CHILD.lock().ok().and_then(|mut g| g.take());
        let output = if let Some(mut c) = child {
            // 检查进程是否还活着
            match c.try_wait() {
                Ok(None) => {
                    // 进程还在运行 — 5s 超时兜底
                    let panic_timeout = Duration::from_secs(5);
                    let deadline = Instant::now() + panic_timeout;
                    let mut exited = false;
                    while Instant::now() < deadline {
                        match c.try_wait() {
                            Ok(Some(status)) => {
                                log::info!("svn_with_progress 子进程退出: {:?}", status.code());
                                exited = true;
                                break;
                            }
                            Ok(None) => thread::sleep(Duration::from_millis(50)),
                            Err(e) => {
                                log::error!("子进程 wait 错误: {}", e);
                                break;
                            }
                        }
                    }
                    if !exited {
                        log::warn!("5s 超时兜底: kill 子进程");
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                    c.wait_with_output().unwrap_or_else(|e| {
                        log::error!("wait_with_output 失败: {}", e);
                        std::process::Output {
                            status: std::process::ExitStatus::default(),
                            stdout: combined_stdout.clone().into_bytes(),
                            stderr: Vec::new(),
                        }
                    })
                }
                Ok(Some(status)) => {
                    // 已在取消操作中被 kill，构造输出
                    std::process::Output {
                        status,
                        stdout: combined_stdout.clone().into_bytes(),
                        stderr: Vec::new(),
                    }
                }
                Err(e) => {
                    log::error!("try_wait 错误: {}", e);
                    std::process::Output {
                        status: std::process::ExitStatus::default(),
                        stdout: combined_stdout.clone().into_bytes(),
                        stderr: Vec::new(),
                    }
                }
            }
        } else {
            log::error!("子进程句柄丢失");
            std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: combined_stdout.clone().into_bytes(),
                stderr: Vec::new(),
            }
        };

        let elapsed = start.elapsed();
        log::info!("svn_with_progress 完成 (耗时: {:?}, 退出码: {:?})", elapsed, output.status.code());

        if output.status.success() || is_cancelled() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("svn_with_progress stderr: {}", stderr);

            if is_auth_error(&stderr) {
                Err(AppError::SvnAuthFailed(stderr.to_string()))
            } else {
                Err(AppError::SvnCommand(stderr.to_string()))
            }
        }
    }).await;

    match result {
        Ok(inner) => {
            let r = inner.map_err(|e| AppError::Repo(format!("spawn_blocking error: {}", e)))?;
            // emit operation:completed
            app_handle.emit("operation:completed", serde_json::json!({
                "result": "success", "detail": ""
            })).ok();
            Ok(r)
        }
        Err(_elapsed) => {
            app_handle.emit("operation:error", serde_json::json!({
                "errorCode": "TIMEOUT", "message": "SVN operation timed out"
            })).ok();
            Err(AppError::Timeout("SVN operation timed out".into()))
        }
    }
}

/// 终止当前 svn 子进程
pub fn kill_current_process() {
    if let Ok(mut guard) = CURRENT_CHILD.lock() {
        if let Some(mut child) = guard.take() {
            log::info!("终止 SVN 进程 pid={:?}", child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// 终止当前 svn 子进程并执行 cleanup
pub async fn kill_and_cleanup(cwd: &str) {
    kill_current_process();
    log::info!("执行 svn cleanup (cwd: {})", cwd);
    let _ = run_svn(&["cleanup"], cwd, None).await;
    set_cancelled(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_empty() {
        assert!(validate_path("").is_err());
    }

    #[test]
    fn test_validate_path_null() {
        assert!(validate_path("path\u{0}name").is_err());
    }

    #[test]
    fn test_validate_path_ok() {
        assert!(validate_path("/tmp/valid/path").is_ok());
    }

    #[test]
    fn test_get_timeout_secs_blame() {
        assert_eq!(get_timeout_secs(&["blame", "--xml"]), 120);
    }

    #[test]
    fn test_get_timeout_secs_default() {
        assert_eq!(get_timeout_secs(&["status", "--xml"]), 60);
    }

    #[test]
    fn test_check_auth_hint_with_keyword() {
        assert!(check_auth_hint("Authentication realm: svn"));
        assert!(!check_auth_hint("normal stderr output"));
    }

    #[test]
    fn test_is_auth_error() {
        assert!(is_auth_error("E170001: authentication"));
        assert!(!is_auth_error("normal stderr"));
    }

    #[test]
    fn test_get_svn_path_not_empty() {
        let path = get_svn_path();
        assert!(!path.as_os_str().is_empty(), "SVN 路径不应为空");
    }

    #[test]
    fn test_cancelled_flag_roundtrip() {
        // 初始状态应为 false
        set_cancelled(false);
        assert!(!is_cancelled());
        // 设置为 true
        set_cancelled(true);
        assert!(is_cancelled());
        // 重置为 false
        set_cancelled(false);
        assert!(!is_cancelled());
    }

    #[test]
    fn test_kill_current_process_no_child() {
        // 无子进程时调用不应 panic
        kill_current_process();
        // 验证清理后 CURRENT_CHILD 为空
        assert!(CURRENT_CHILD.lock().unwrap().is_none());
    }

    #[test]
    fn test_extract_host_valid() {
        assert_eq!(extract_host("svn://example.com/repo").unwrap(), "example.com");
        assert_eq!(extract_host("https://svn.example.com:443/path").unwrap(), "svn.example.com:443");
        assert_eq!(extract_host("svn+ssh://192.168.1.1/project").unwrap(), "192.168.1.1");
        assert_eq!(extract_host("http://host").unwrap(), "host");
        assert!(extract_host("invalid-url").is_err());
        assert!(extract_host("ftp://host/path").is_err());
    }

    #[test]
    fn test_extract_realm_various() {
        let with_realm = "Warning: some output\nAuthentication realm: <svn://example.com> SVN Repository\nmore output";
        assert!(extract_realm(with_realm).contains("Authentication realm"));

        let no_realm = "some error output";
        assert_eq!(extract_realm(no_realm), "some error output");

        let empty = "";
        assert_eq!(extract_realm(empty), "unknown auth error");
    }
}
