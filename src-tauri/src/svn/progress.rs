use crate::models::error::AppError;
use crate::svn::executor::{
    get_svn_env, get_svn_path, is_cancelled, is_auth_error, BASE_SVN_ARGS, CURRENT_CHILD,
};
use log;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::task::spawn_blocking;
use walkdir::WalkDir;

/// 格式化字节速度为人类可读字符串
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000_000.0 {
        format!("{:.1} GB/s", bytes_per_sec / 1_000_000_000.0)
    } else if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

/// 带进度推送的 SVN 命令执行。
///
/// 与 `run_svn` 不同，此函数通过双管道逐行读取 stdout/stderr，
/// 实时解析进度并通过 Tauri event 推送到前端。
/// 使用场景：checkout / update / commit / switch / merge / export
///
/// `target_dir`：可选，指定后跟踪已完成文件的实际磁盘大小用于计算字节传输速度
pub async fn run_svn_with_progress(
    args: &[&str],
    cwd: &str,
    credentials: Option<&crate::svn::types::SvnCredentials>,
    app_handle: tauri::AppHandle,
    operation: &str,
    target_dir: Option<&str>,
) -> Result<String, AppError> {
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    // 1. emit operation:started
    app_handle.emit("operation:started", serde_json::json!({
        "operation": operation
    })).ok();

    let svn_path = get_svn_path();
    let svn_path_str = svn_path.to_string_lossy().to_string();

    // 构建 args — 移除 --non-interactive 让 svn 输出进度信息（速度/百分比）
    // 进度行格式: "   45%   1234K   1.2MB/s   00:12"
    let mut all_args: Vec<String> = Vec::new();
    for s in BASE_SVN_ARGS.iter() {
        if *s != "--non-interactive" {
            all_args.push(s.to_string());
        }
    }
    for arg in args.iter() {
        if arg != &"--non-interactive" {
            all_args.push(arg.to_string());
        }
    }

    let cwd_str = cwd.to_string();
    let creds = credentials.cloned();
    let operation_owned = operation.to_string();
    let target_dir_owned = target_dir.map(|s| s.to_string());
    let ah = app_handle.clone();

    let result = spawn_blocking(move || {
        let start = Instant::now();
        let mut was_cancelled = false;
        let target_dir = target_dir_owned.as_deref();

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
                    (Err(AppError::SvnNotFound), false)
                } else {
                    (Err(AppError::Io(e)), false)
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
        // 注意：SVN 进度行以 \r 结尾（原地覆写），非进度行以 \n 结尾。
        // 因此不能使用 BufReader::lines()（只按 \n 分割），改为字节级读取按 \r/\n 双分割。
        let tx_e = tx_stderr.clone();
        let tx_p2 = tx_panic.clone();
        let op_e = operation_owned.clone();
        let stderr_thread = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(handle) = stderr_handle {
                    let mut reader = BufReader::new(handle);
                    let mut line_buf = Vec::new();
                    loop {
                        if is_cancelled() {
                            break;
                        }
                        let mut byte = [0u8; 1];
                        match reader.read(&mut byte) {
                            Ok(0) => break, // EOF
                            Ok(_) => {
                                let b = byte[0];
                                if b == b'\n' || b == b'\r' {
                                    if !line_buf.is_empty() {
                                        let text = String::from_utf8_lossy(&line_buf).to_string();
                                        line_buf.clear();
                                        let _ = tx_e.send(text);
                                    }
                                } else {
                                    line_buf.push(b);
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                            Err(_) => break,
                        }
                    }
                    // 发送缓冲区剩余的未闭合行
                    if !line_buf.is_empty() {
                        let text = String::from_utf8_lossy(&line_buf).to_string();
                        let _ = tx_e.send(text);
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
        // 缓存上一次解析到的速度，心跳发射时保持该值不让前端闪烁消失
        let mut last_known_speed: Option<String> = None;

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
        // SVN 进度行格式: "...  45%"  或   "...  45%   1234K   1.2MB/s   00:12"
        let extract_percentage = |line: &str| -> Option<u8> {
            let trimmed = line.trim_start();
            // 去掉前导的 "..."
            let content = if trimmed.starts_with("...") {
                trimmed.trim_start_matches("...").trim_start()
            } else {
                trimmed
            };
            // 找到 % 位置并提取前面的数字
            if let Some(pct_end) = content.find('%') {
                content[..pct_end].trim().parse::<u8>().ok()
            } else {
                None
            }
        };

        // 从 stderr 行提取传输速度和已耗时间
        fn extract_speed(line: &str) -> (Option<String>, Option<String>) {
            // SVN 进度行格式示例:
            //   "   45%   1234K   1.2MB/s   00:12"
            //   "   85%   2.3MB/s   00:05"
            let trimmed = line.trim_start();
            // 跳过百分比部分（如 "45%" 或 "... 45%"）
            let after_pct = if trimmed.starts_with("...") {
                trimmed.trim_start_matches("...").trim_start()
            } else {
                trimmed
            };
            // 找到 % 位置，取之后的部分
            let rest = match after_pct.find('%') {
                Some(pos) => after_pct[pos + 1..].trim(),
                None => return (None, None),
            };
            if rest.is_empty() {
                return (None, None);
            }
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let mut speed: Option<String> = None;
            let mut elapsed: Option<String> = None;
            for part in &parts {
                if part.ends_with("/s") || part.ends_with("/S") {
                    speed = Some(part.to_string());
                } else if part.contains(':') {
                    elapsed = Some(part.to_string());
                }
            }
            (speed, elapsed)
        }

        loop {
            // 检查取消
            if is_cancelled() {
                was_cancelled = true;
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
                                let (speed_str, elapsed_str) = extract_speed(&line);
                                if speed_str.is_some() {
                                    last_known_speed = speed_str.clone();
                                }
                                let progress = crate::svn::types::OperationProgress {
                                    operation: operation_owned.clone(),
                                    percent: pct,
                                    stage: "processing".into(),
                                    file_count,
                                    completed_count,
                                    pending_count: 0,
                                    speed: speed_str,
                                    elapsed: elapsed_str,
                                    current_lines: vec![],
                                };
                                ah.emit("operation:progress", &progress).ok();
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

            // 定期进度推送（500ms 节流，不依赖 stderr 百分比行）
            // 即使 SVN 在管道模式下不输出进度行，也能统计已完成的文件数
            if file_count > 0 || (!stdout_done || !stderr_done) {
                let now = Instant::now();
                if last_progress_time.elapsed() >= Duration::from_millis(500) {
                    last_progress_time = now;
                    let elapsed = now.duration_since(start);
                    let elapsed_secs = elapsed.as_secs_f64();
                    let elapsed_str = Some(format!(
                        "{:02}:{:02}", elapsed.as_secs() / 60, elapsed.as_secs() % 60
                    ));
                    // 速度（优先级）：stderr 解析速度 > 字节传输速度 > 文件完成率
                    let byte_speed = if elapsed_secs > 0.0 {
                        target_dir.and_then(|td| {
                            let size: u64 = WalkDir::new(td)
                                .into_iter()
                                .filter_map(|e| e.ok())
                                .filter(|e| e.file_type().is_file())
                                .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                                .sum();
                            if size > 0 {
                                Some(format_speed(size as f64 / elapsed_secs))
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };
                    let display_speed = last_known_speed.clone().or(byte_speed).or_else(|| {
                        if completed_count > 0 && elapsed_secs > 0.0 {
                            Some(format!("{:.1} files/s", completed_count as f64 / elapsed_secs))
                        } else {
                            None
                        }
                    });
                    let progress = crate::svn::types::OperationProgress {
                        operation: operation_owned.clone(),
                        percent: 0,
                        stage: "processing".into(),
                        file_count,
                        completed_count,
                        pending_count: 0,
                        speed: display_speed,
                        elapsed: elapsed_str,
                        current_lines: vec![],
                    };
                    ah.emit("operation:progress", &progress).ok();
                }
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

        if output.status.success() || was_cancelled {
            (Ok(String::from_utf8_lossy(&output.stdout).to_string()), was_cancelled)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("svn_with_progress stderr: {}", stderr);

            if is_auth_error(&stderr) {
                (Err(AppError::SvnAuthFailed(stderr.to_string())), was_cancelled)
            } else {
                (Err(AppError::SvnCommand(stderr.to_string())), was_cancelled)
            }
        }
    }).await;

    match result {
        Ok((inner, was_cancelled)) => {
            let r = inner.map_err(|e| AppError::Repo(format!("spawn_blocking error: {}", e)))?;
            if was_cancelled {
                // 取消后不再 emit 新事件（cancelled 事件已由 cancel.rs 推送）
                return Ok(r);
            }
            // emit operation:completed
            app_handle.emit("operation:completed", serde_json::json!({
                "result": "success", "detail": ""
            })).ok();
            Ok(r)
        }
        Err(join_err) => {
            app_handle.emit("operation:error", serde_json::json!({
                "errorCode": "TIMEOUT", "message": "SVN operation timed out"
            })).ok();
            Err(AppError::Repo(format!("spawn_blocking error: {}", join_err)))
        }
    }
}
