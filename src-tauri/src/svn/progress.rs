use crate::models::error::AppError;
use crate::svn::executor::{
    get_svn_env, get_svn_path, is_cancelled, is_auth_error, BASE_SVN_ARGS, CURRENT_CHILD,
};
use log;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::task::spawn_blocking;

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
        let mut was_cancelled = false;

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
