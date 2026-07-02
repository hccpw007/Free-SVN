# 01-总则与项目结构

## 技术栈

| 层面 | 技术选型 |
|------|----------|
| 语言 | Rust (edition 2021) |
| 框架 | Tauri 2.x |
| 包管理 | Cargo |
| 日志 | tauri-plugin-log + log crate |
| 持久化 | tauri-plugin-store |
| 序列化 | serde + serde_json |
| 错误处理 | thiserror |
| 异步 | tokio（Tauri 内置） |

## 项目结构

```
src-tauri/src/
├── main.rs                        # 二进制入口，仅调用 lib::run()
├── lib.rs                         # Tauri 应用入口，注册插件和命令
│
├── commands/                      # Tauri Command 定义（前端可调用的接口）
│   ├── mod.rs                     #   command 模块导出
│   ├── status.rs                  #   status / info
│   ├── commit.rs                  #   commit
│   ├── update.rs                  #   update
│   ├── checkout.rs                #   checkout
│   ├── log.rs                     #   log / blame
│   ├── diff.rs                    #   diff
│   ├── file_ops.rs                #   add / delete / revert / resolve
│   ├── branch_ops.rs              #   copy(分支/标签) / switch / merge
│   ├── ignore.rs                  #   ignore (propget/propset svn:ignore)
│   ├── cleanup.rs                 #   cleanup / export
│   ├── relocate.rs                #   relocate
│   ├── lock.rs                    #   lock / unlock
│   ├── cancel.rs                  #   cancel_operation（取消当前操作）
│   └── logs.rs                    #   get_logs / export_logs
│
├── svn/                           # SVN CLI 执行核心
│   ├── mod.rs                     #   SVN CLI 执行核心模块入口
│   ├── executor.rs                #   CLI 调用 + 超时 + 错误处理 + 取消检测
│   ├── parser.rs                  #   XML 输出解析
│   ├── queue.rs                   #   操作队列与并发控制（Mutex 封装）
│   └── types.rs                   #   共享数据类型
│
├── models/                        # 数据模型（结构体、枚举）
│   ├── mod.rs
│   ├── error.rs                   #   AppError 定义
│   ├── repo.rs                    #   仓库模型
│   └── file.rs                    #   文件/变更模型
│
├── services/                      # 业务逻辑层（目前为空，SVN 逻辑在 svn/ 模块中）
│   └── mod.rs
│
├── tools/                          # 外部工具调用（差异工具/合并工具/工具路径检测）
│   ├── mod.rs
│   ├── differ.rs                   #   外部差异工具
│   ├── merger.rs                   #   外部合并工具
│   └── resolver.rs                 #   工具路径检测与回退
│
├── shell_integration/             # 右键菜单管理
│   ├── mod.rs
│   ├── macos.rs                   #   macOS Finder 扩展
│   ├── windows.rs                 #   Windows Shell 扩展
│   └── linux.rs                   #   Linux 文件管理器脚本
│
├── logging/                       # 日志模块
│   ├── mod.rs
│   └── logger.rs                  #   日志写入 + 轮转
│
├── config/                        # 配置持久化
│   ├── mod.rs
│   └── store.rs                   #   tauri-plugin-store 封装（含损坏恢复）
│
├── capabilities/
│   └── default.json               # 权限声明
├── icons/                         # 应用图标
├── Cargo.toml
├── build.rs
└── tauri.conf.json
```

## Cargo.toml 关键依赖

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }  # 系统托盘支持
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tauri-plugin-store = "2"
tauri-plugin-single-instance = "2"  # 单实例防护
tauri-plugin-log = "2"              # 日志
log = "0.4"
tauri-plugin-window-state = "2"     # 窗口状态持久化
```

## 分层职责

```
commands/             → 接口层：参数校验、调用 service、返回结果（薄层，不写业务逻辑）
svn/                  → SVN 核心：CLI 执行、XML 解析、并发控制
  executor.rs         →   CLI 调用 + 超时 + 取消检测
  parser.rs           →   XML 输出解析
  queue.rs            →   操作队列与 Mutex 并发控制
  types.rs            →   共享数据类型
models/               → 数据层：类型定义、序列化（纯数据结构，无方法）
services/             → 业务层（预留，复杂逻辑可放此处）
shell_integration/    → 右键菜单注册/清理
logging/              → 日志写入和轮转
config/               → 配置持久化与损坏恢复
lib.rs                → 应用入口：注册 plugin 和 command
```

禁止反向依赖：`svn/` 不能 import `commands/`；`logging/` 不能 import `commands/`。

---

# 02-Tauri Command 规范

## 命令定义

每个 Command 放在独立的 rs 文件中，按业务领域划分。

```rust
// commands/status.rs
use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;

#[derive(Deserialize)]
pub struct StatusParams {
    pub path: String,
}

#[tauri::command]
pub async fn get_status(params: StatusParams) -> Result<SvnStatusOutput, AppError> {
    if params.path.is_empty() {
        return Err(AppError::InvalidInput("路径不能为空".into()));
    }
    // Command 只做参数校验 + 调用 svn 模块，不做业务处理
    svn::executor::status(&params.path).await
}
```

## 命令清单

每个操作对应一个 Tauri Command（参见业务设计文档 §6.1）：

| 操作 | 输入参数 | Command 文件 |
|------|---------|-------------|
| status | path | `commands/status.rs` |
| info | path | `commands/status.rs` |
| checkout | url, targetPath, depth, ignoreExternals | `commands/checkout.rs` |
| commit | paths[], message, keepLocks | `commands/commit.rs` |
| update | path, revision?, depth?, ignoreExternals? | `commands/update.rs` |
| log | path, limit, revision?, search? | `commands/log.rs` |
| diff | path, revision1?, revision2? | `commands/diff.rs` |
| add | path[] | `commands/file_ops.rs` |
| delete | path[], keepLocal? | `commands/file_ops.rs` |
| revert | path[] | `commands/file_ops.rs` |
| resolve | path, resolution | `commands/file_ops.rs` |
| switch | path, targetUrl, ignoreAncestry? | `commands/branch_ops.rs` |
| copy (branch/tag) | srcUrl, dstUrl, message, revision? | `commands/branch_ops.rs` |
| merge | srcUrl, revStart, revEnd, targetPath | `commands/branch_ops.rs` |
| cleanup | path | `commands/cleanup.rs` |
| export | path, targetDir, revision?, ignoreExternals? | `commands/cleanup.rs` |
| ignore | path, pattern | `commands/ignore.rs` |
| blame | path, revision? | `commands/log.rs` |
| relocate | path, fromUrl, toUrl | `commands/relocate.rs` |
| property | path, propName?, action? | `commands/ignore.rs` |
| lock | path[], message? | `commands/lock.rs` |
| unlock | path[] | `commands/lock.rs` |
| cancel_operation | 无（取消当前操作） | `commands/cancel.rs` |
| get_logs | 无 | `commands/logs.rs` |
| export_logs | target_path | `commands/logs.rs` |

## Tauri Event 协议

长操作（checkout/update/export/merge）通过 Tauri event 向前端推送进度。以下是完整事件协议：

| 事件名 | 方向 | Payload | 触发时机 |
|--------|------|---------|---------|
| `operation:started` | 后端 → 前端 | `{ operation: "checkout" }` | 长操作开始时 |
| `operation:progress` | 后端 → 前端 | `{ percent: 0-100, stage: "检出中...", fileCount: 42 }` | 每处理 N 个文件或阶段变更时 |
| `operation:completed` | 后端 → 前端 | `{ result: "success"|"cancelled"|"error", detail: "..." }` | 操作正常完成/取消/失败 |
| `operation:error` | 后端 → 前端 | `{ errorCode: "SVN_TIMEOUT", message: "..." }` | 操作过程中发生错误 |

**Payload 类型定义（前后端同步）：**

```ts
// 前端: types/svn.ts
interface OperationProgress {
  operation: string    // checkout | update | export | merge
  percent: number      // 0-100
  stage: string        // 当前阶段描述
  fileCount: number    // 已处理文件数
}

interface OperationResult {
  result: 'success' | 'cancelled' | 'error'
  detail?: string
}
```

```rust
// 后端: svn/types.rs
#[derive(Serialize)]
pub struct OperationProgress {
    pub operation: String,
    pub percent: u8,
    pub stage: String,
    pub file_count: u32,
}
```

**前端监听：**
```ts
// services/svn.ts
import { listen } from '@tauri-apps/api/event'

listen<OperationProgress>('operation:progress', (event) => {
  // 更新 ProgressOverlay
})

## Command 规范

- **Command 函数必须是异步的**（`async fn`）
- 参数较多时（≥2 个）使用 `#[derive(Deserialize)]` 结构体，避免逐个参数传递
- 返回值统一使用 `Result<T, AppError>`，`AppError` 通过 `#[error("ERROR_CODE: {0}")]` 返回**错误码 key**（非中文文字）
- 一个 Command 只做一件事，不做"万能接口"
- Command 内部只做参数校验 + 调用 svn 模块，不做业务处理
- 所有与服务端通信的 Command（log/diff/update/checkout/switch/merge/copy/export 等）**执行前调用网络检测**；`status` 为纯本地操作，不检测网络
- 长操作（checkout/update/export/merge）执行前检查取消标志

## 注册命令

```rust
// lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init())  // 单实例防护
        .plugin(tauri_plugin_window_state::default())   // 窗口状态持久化
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // 初始化日志
            logging::logger::init(app.handle())?;
            // 初始化配置存储
            config::store::init(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::status::get_status,
            commands::status::get_info,
            commands::commit::create_commit,
            commands::update::update_workspace,
            commands::checkout::checkout_repo,
            commands::log::get_log,
            commands::log::get_blame,
            commands::diff::get_diff,
            commands::file_ops::add_files,
            commands::file_ops::delete_files,
            commands::file_ops::revert_files,
            commands::file_ops::resolve_conflict,
            commands::branch_ops::switch_branch,
            commands::branch_ops::copy_branch_tag,
            commands::branch_ops::merge_branch,
            commands::ignore::set_ignore,
            commands::cleanup::cleanup_workspace,
            commands::cleanup::export_workspace,
            commands::relocate::relocate_repo,
            commands::lock::lock_files,
            commands::lock::unlock_files,
            commands::cancel::cancel_operation,
            commands::logs::get_logs,
            commands::logs::export_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 系统托盘与右键菜单注册

```rust
// lib.rs setup 中
app.on_tray_event(|app, event| {
    match event {
        TrayIconEvent::Click(_) => { /* 显示主窗口 */ }
        _ => {}
    }
});
```

---

# 03-错误处理规范（thiserror）

使用 `thiserror` 定义 `AppError`，Command 返回 `Result<T, AppError>`，Tauri 自动通过 `Display` 序列化错误给前端。

**⚡ 错误码规则（重要）：** 后端返回**错误码 key**（如 `SVN_TIMEOUT`）而非中文文字。前端根据当前 locale 将错误码渲染为用户界面语言。仅在日志中记录原始 stderr 供诊断。

```rust
// Cargo.toml 依赖
// thiserror = "2"

// models/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    /// SVN 命令执行失败
    #[error("SVN_EXEC_FAILED: {0}")]
    SvnCommand(String),

    /// SVN XML 输出解析失败
    #[error("SVN_PARSE_FAILED: {0}")]
    ParseFailed(String),

    /// 参数校验失败
    #[error("INVALID_INPUT: {0}")]
    InvalidInput(String),

    /// 文件系统错误
    #[error("IO_ERROR: {0}")]
    Io(#[from] std::io::Error),

    /// 仓库操作异常
    #[error("REPO_ERROR: {0}")]
    Repo(String),

    /// 操作超时
    #[error("SVN_TIMEOUT: {0}")]
    Timeout(String),

    /// 内置 svn 未找到/不可执行
    #[error("SVN_NOT_FOUND")]
    SvnNotFound,

    /// 不是 SVN 工作副本
    #[error("SVN_NOT_WORKING_COPY")]
    NotWorkingCopy,

    /// 有写操作正在进行中
    #[error("SVN_OPERATION_IN_PROGRESS")]
    OperationInProgress,

    /// 操作被用户取消
    #[error("SVN_CANCELLED")]
    Cancelled,

    /// 网络不可达（服务端操作前检测）
    #[error("NETWORK_UNREACHABLE")]
    NetworkUnreachable,
}
```

## 统一错误码目录

以下为完整错误码表，后端 Rust 枚举和前端 `types/error-codes.ts` 同步维护：

| 错误码 | Rust 枚举变体 | 含义 | 前端行为 |
|--------|-------------|------|---------|
| `SVN_EXEC_FAILED` | `SvnCommand` | SVN CLI 执行返回非零退出码 | 显示错误信息 + 日志详情 |
| `SVN_PARSE_FAILED` | `ParseFailed` | SVN XML 输出解析失败 | 显示"解析错误" |
| `INVALID_INPUT` | `InvalidInput` | 参数校验不通过 | 显示具体校验提示 |
| `IO_ERROR` | `Io` | 文件系统错误 | 显示"文件系统错误" |
| `REPO_ERROR` | `Repo` | 仓库操作异常 | 显示"仓库异常" |
| `SVN_TIMEOUT` | `Timeout` | SVN 命令超时（默认 60s） | 显示"操作超时，请重试" |
| `SVN_NOT_FOUND` | `SvnNotFound` | 内置 svn 未找到/不可执行 | 显示"svn 未找到，请重新安装 Free-SVN" |
| `SVN_NOT_WORKING_COPY` | `NotWorkingCopy` | 路径不是 SVN 工作副本 | 显示"不是 SVN 工作副本" |
| `SVN_OPERATION_IN_PROGRESS` | `OperationInProgress` | 写操作进行中，冲突 | 显示"操作进行中" |
| `SVN_CANCELLED` | `Cancelled` | 用户取消操作 | 显示"操作已取消" |
| `NETWORK_UNREACHABLE` | `NetworkUnreachable` | 网络不可达 | 禁用服务端按钮 + 显示"网络不可用" |

**规则：** 前端 `types/error-codes.ts` 中定义与后端完全相同的错误码常量集合，每个错误码对应一条 i18n 翻译 key。

### IPC 参数类型同步守则

所有 Tauri Command 的**参数结构体和返回类型**必须在 Rust 侧和 TypeScript 侧同步定义：

```rust
// Rust: commands/status.rs
#[derive(Deserialize)]
pub struct StatusParams {
    pub path: String,
}
```

```ts
// TypeScript: types/svn.ts
export interface StatusParams {
  path: string
}
```

**规则：** 修改任何 Command 的签名（参数/返回值）时，必须同步更新另一侧的类型定义。若类型定义不一致导致运行时错误，视为 bug。

---

# 04-SVN 操作规范

SVN 操作统一封装在 `svn/` 模块中，**禁止**散布 `Command::new("svn")` 调用。

## 模块结构

```
svn/
├── mod.rs               # 模块入口，pub use 重导出
├── executor.rs          # CLI 调用 + 超时 + 错误处理 + 取消检测
├── parser.rs            # XML 输出解析（serde 反序列化）
├── queue.rs             # 操作队列与并发控制（Mutex 封装）
└── types.rs             # 共享数据类型
```

## Executor — CLI 调用核心

```rust
// svn/executor.rs
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use log;
use tokio::time::{timeout, Duration};
use tokio::task::spawn_blocking;
use crate::models::error::AppError;

/// 全局取消标志
pub static CANCELLED: AtomicBool = AtomicBool::new(false);

/// 当前 svn 子进程的句柄（用于取消时 kill）
pub static CURRENT_CHILD: Mutex<Option<std::process::Child>> = ...;

/// 检测取消信号
pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}

/// 同步执行 SVN 命令（在 spawn_blocking 中运行）
fn run_svn_sync(args: &[&str], cwd: &str) -> Result<String, AppError> {
    log::info!("svn {} (cwd: {})", args.join(" "), cwd);

    let mut child = Command::new(get_svn_path())
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;

    // 保存子进程句柄用于取消
    *CURRENT_CHILD.lock().unwrap() = Some(child.try_wait().ok());

    let output = child.wait_with_output().map_err(AppError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::SvnCommand(stderr.to_string()))
    }
}

/// 异步执行 SVN 命令（带超时控制 + 取消检测）
async fn run_svn(args: &[&str], cwd: &str) -> Result<String, AppError> {
    // 取消检测
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = cwd.to_string();

    timeout(Duration::from_secs(get_timeout_secs()), spawn_blocking(move || {
        run_svn_sync(&args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(), &cwd)
    }))
    .await
    .map_err(|_| AppError::Timeout("SVN 命令执行超时".into()))?
    .map_err(|e| match e {
        AppError::Cancelled => AppError::Cancelled,
        _ => e,
    })?
}
```

## Queue — 操作队列与并发控制

```rust
// svn/queue.rs
use std::sync::Mutex;
use tokio::task::JoinHandle;

pub struct SvnQueue {
    /// 当前写操作的句柄（同一工作副本的写操作互斥）
    write_lock: Mutex<Option<JoinHandle<()>>>,
}

impl SvnQueue {
    /// 尝试获取写操作锁
    pub fn try_lock(&self) -> Result<(), AppError> {
        let mut guard = self.write_lock.lock().unwrap();
        if guard.is_some() {
            Err(AppError::OperationInProgress)
        } else {
            *guard = None; // 标记占用
            Ok(())
        }
    }

    /// 释放写操作锁
    pub fn unlock(&self) {
        *self.write_lock.lock().unwrap() = None;
    }
}

/// 分类：
/// - 只读操作（可并发）：status, info, diff, log, blame
/// - 写操作（互斥）：commit, update, switch, merge, checkout, cleanup, revert, resolve, add, delete, ignore, lock, unlock, copy, export, relocate, property
/// - 取消操作（特殊，不经过锁）：cancel_operation
```

## 长操作取消模式

```rust
// commands/cancel.rs
#[tauri::command]
pub async fn cancel_operation(state: State<'_, AppState>) -> Result<(), AppError> {
    // 1. 设置取消标志
    svn::executor::CANCELLED.store(true, Ordering::SeqCst);

    // 2. 终止当前 svn 子进程
    if let Some(mut child) = svn::executor::CURRENT_CHILD.lock().unwrap().take() {
        // 发送 SIGTERM（Unix）/ TerminateProcess（Windows）
        child.kill().ok();
        child.wait().ok();
    }

    // 3. 执行 svn cleanup 恢复工作副本状态
    let path = state.current_path.lock().unwrap().clone();
    if let Some(p) = path {
        svn::executor::run_svn(&["cleanup"], &p).await.ok();
    }

    // 4. 重置取消标志
    svn::executor::CANCELLED.store(false, Ordering::SeqCst);

    // 5. 通知前端（通过 Tauri event）
    state.app_handle.emit("operation:completed", "已取消").ok();

    Ok(())
}
```

## 网络可达性检测

```rust
/// 检测网络是否可达（使用 tokio::net::TcpStream 尝试连接 SVN 服务器）
/// 仅对有服务端通信的操作执行，status（纯本地）不检测
pub async fn check_network(server_url: &str) -> Result<(), AppError> {
    let host = extract_host(server_url)?;
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect((host.as_str(), 3690))
    ).await {
        Ok(Ok(_)) => Ok(()),
        _ => Err(AppError::NetworkUnreachable),
    }
}
```

## SVN 操作原则

- 所有 SVN 子进程调用走 `svn::executor::run_svn`，禁止散布 `Command::new("svn")`
- 优先 `--xml` 参数获取结构化输出，用 `svn::parser` 中的 serde 反序列化
- 每次执行必须设置超时（默认 60 秒）
- 禁止字符串正则匹配解析 SVN 输出
- 执行前后必须记录日志（命令 + 参数 + 耗时）
- 写操作通过 `svn::queue::SvnQueue` 互斥（同一工作副本）
- 长操作循环中检测 `svn::executor::is_cancelled()`
- 服务端操作前调用网络检测；status（纯本地）跳过检测
- 使用内置 svn 可执行文件路径（通过 Tauri `resource_dir()` 获取），不依赖系统 svn

---

## 外部工具执行规范

差异/合并工具（VS Code、Beyond Compare、Kaleidoscope 等）的调用封装在 `tools/` 模块中，与 SVN 命令独立：

```
tools/
├── mod.rs               # 工具模块入口
├── differ.rs            # 外部差异工具调用
├── merger.rs            # 外部合并工具调用
└── resolver.rs          # 工具路径检测与回退
```

```rust
// tools/differ.rs
pub fn open_external_diff(tool: &str, params: ExternalDiffParams) -> Result<(), AppError> {
    // 与 svn::executor::run_svn 不同：
    // 1. 不设置超时（用户可能长时间编辑）
    // 2. stdout/stderr 不捕获（输出到工具窗口）
    // 3. 返回工具进程的退出码
    // 4. 执行前检测工具是否已安装
}
```

**外部工具调用与 SVN 调用的区别：**

| 特性 | SVN 执行 | 外部工具执行 |
|------|---------|-------------|
| 超时 | 60 秒默认 | 不设置超时 |
| 输出捕获 | 捕获 stdout/stderr | 不捕获（窗口工具）|
| 取消支持 | 可取消 | 不取消 |
| 进程等待 | 等待完成 | 不等待（分离进程）|
| 执行检查 | 内置 svn 路径 | 检测工具是否已安装 |

**工具回退策略：** 当用户配置的外部工具未安装或执行失败时，自动回退到内置 diff2html，通过 toast 提示"已回退到内置差异工具"。

---

## 内置二进制工具管理

bundled svn 二进制统一通过 `tools/` 模块管理。当前仅有 svn，后续可能增加其他工具。

```rust
// tools/mod.rs — 所有捆绑二进制文件的统一管理入口
pub fn get_svn_path() -> PathBuf {
    // 通过 Tauri resource_dir() 获取内置 svn 路径
    // macOS: resources/svn/macos/svn
    // Windows: resources/svn/win/svn.exe
    // Linux: resources/svn/linux/svn
}

pub fn get_svn_version() -> Result<String, AppError> {
    // 执行 svn --version --quiet 获取版本号
}

pub fn validate_svn() -> Result<(), AppError> {
    // 启动时检测 svn 是否可执行
    // 不可执行时返回 SvnNotFound
}
```

**SVN 版本信息：** 底部状态栏和设置页关于区显示内置 svn 版本号（通过 `svn --version --quiet` 获取）。

**许可证声明：** 应用关于页面和发行说明中声明 Apache 2.0 许可证和 Subversion 版权归属。

---

# 05-安全规范

## 路径安全

```rust
fn validate_path(path: &str) -> Result<&str, AppError> {
    if path.contains('\0') {
        return Err(AppError::InvalidInput("路径包含非法字符".into()));
    }
    Ok(path)
}
```

- `Command::new("svn").args(...)` 不经过 shell，天然防注入
- 禁止使用 `Command::new("sh").arg("-c")` 执行 SVN 命令
- 不在日志中输出密码、token 等敏感信息

---

# 05.1-日志规范

## 日志位置

使用 `tauri::api::path::app_log_dir()` 获取：
- macOS: `~/Library/Logs/com.free-svn/`
- Windows: `%APPDATA%/Free-SVN/logs/`
- Linux: `~/.local/share/Free-SVN/logs/`

## 日志文件

| 文件 | 说明 |
|------|------|
| `free-svn.log` | 当前日志 |
| `free-svn.log.1` | 轮转备份 1 |
| `free-svn.log.2` | 轮转备份 2 |

## 日志级别

| 级别 | 记录内容 |
|------|---------|
| ERROR | SVN 命令执行失败、解析失败、超时、未捕获异常 |
| WARN | 非致命问题（store 损坏已恢复、右键菜单注册失败） |
| INFO | SVN 命令执行（命令+参数+耗时）、操作开始/完成、窗口事件 |
| DEBUG | 原始 svn stdout/stderr、Tauri event 详情、状态变更日志 |

## 日志轮转

- 单文件最大 5MB
- 保留最近 3 个文件
- 应用启动时检查并轮转

## 日志导出

```rust
#[tauri::command]
fn get_logs() -> Result<String, AppError>       // 返回日志内容
#[tauri::command]
fn export_logs(target_path: String) -> Result<(), AppError>  // 导出日志到指定路径
```

设置页 → 关于 → [导出日志] 按钮触发，导出包含：日志文件 + 应用版本 + 系统信息 + 内置 svn 版本

---

# 05.2-配置存储规范

## 存储内容

使用 `tauri-plugin-store` 存储，序列化为 JSON：

```json
{
  "recentWorkspaces": ["/path/to/project1", ...],
  "currentWorkspace": "/path/to/project1",
  "settings": {
    "defaultCheckoutDir": "~/Documents",
    "globalIgnorePattern": "*.o *.lo *.la",
    "diffTool": "builtin",
    "mergeTool": "builtin",
    "showUnversioned": true,
    "language": "system",
    "autoStart": false
  },
  "windowState": { "x": 100, "y": 100, "width": 1200, "height": 800, "maximized": false }
}
```

## recentWorkspaces 限制

最多保留 20 条记录，超出时移除最早加入的。

## 存储损坏恢复

```rust
// config/store.rs
pub fn load_settings() -> Settings {
    match store.inner().try_deserialize::<Settings>() {
        Ok(s) => s,
        Err(_) => {
            log::warn!("store 损坏，已重置为默认配置");
            backup_corrupted_store(); // 备份为 .store.json.bak
            reset_to_default()        // 重置为默认
        }
    }
}
```

读取 store 时使用 `try_deserialize`，失败时备份损坏文件 + 重置为默认配置。应用正常启动不崩溃。

## 跨版本升级

store 中不存在的字段采用默认值，不做 migration（tauri-plugin-store 自然处理）。

---

# 05.3-Shell 集成规范

## 右键菜单注册

```rust
// shell_integration/macos.rs — macOS Finder 扩展注册
// shell_integration/windows.rs — Windows 注册表 Shell 扩展
// shell_integration/linux.rs — Nautilus 脚本 / Thunar UCA
```

| 平台 | 实现方式 |
|------|----------|
| macOS | Finder 扩展 / Quick Action / URL scheme 回调 |
| Windows | 注册表 Shell 扩展（GUID 注册） |
| Linux | Nautilus 脚本 / Thunar UCA 配置 |

## 右键菜单内容

**SVN 工作副本目录上（完整菜单）：**

```
SVN 提交...              → 打开主窗口提交面板
SVN 更新                 → 后台执行 + 系统通知
SVN 查看日志             → 打开主窗口日志视图
SVN 更新到版本...        → 打开版本选择对话框
───────────────────────────────────────────
SVN 差异对比             → 打开差异视图
SVN 还原...              → 确认后还原
───────────────────────────────────────────
SVN 切换(Switch)...      → 打开切换分支对话框
SVN 分支/标签...         → 打开分支/标签创建对话框
SVN 合并...              → 打开合并向导
───────────────────────────────────────────
SVN 清理                 → 后台执行 + 系统通知
SVN 导出...              → 选择目录后导出
```

普通文件夹/空目录：仅 "SVN 检出..."

## 单实例与右键菜单交互

- 使用 `tauri-plugin-single-instance` 确保单实例运行
- 第二实例携带右键菜单参数 → 传递给首实例处理
- 行为：主窗口最小化到托盘 → 激活并前置；携带工作副本参数 → 切换

## 系统托盘

- 关闭窗口 → 最小化到系统托盘（不退出）
- 托盘右键菜单：显示主窗口 / 更新/清理 / 关于 / 退出
- 后台操作完成 → 系统通知（macOS Notification Center / Windows Toast / Linux notify-send）

---

# 06-编码规范

## 错误处理与日志规范

| 规则 | 约束 |
|------|------|
| 缩进 | 4 空格 |
| 命名 | 结构体/枚举 PascalCase，函数/变量 snake_case |
| 不可变性 | 优先使用 `let`，仅在必要时用 `let mut` |
| 错误处理 | 使用 `Result<T, E>`，禁止 `unwrap()` 和 `expect()`（仅测试可用） |
| 错误日志 | 每个 `map_err` 的错误消息必须包含可诊断的描述信息，禁止空字符串或固定消息 |
| 可见性 | 优先 `pub(crate)`，不暴露不必要的外部 API |

### 错误日志规则

所有 `Err` 变体必须包含可诊断的信息，禁止无上下文错误：

```rust
// ✅ 正确：map_err 包含描述信息
cmd.spawn().map_err(|e| {
    if e.kind() == std::io::ErrorKind::NotFound {
        AppError::SvnNotFound
    } else {
        AppError::Io(e)
    }
})?;

inner.map_err(|e| AppError::Repo(format!("spawn_blocking error: {}", e)))?;

// ✅ 正确：使用 log::warn! / log::error! 记录诊断信息
if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
} else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    log::warn!("svn stderr: {}", stderr);
    Err(AppError::SvnCommand(stderr.to_string()))
}

// ❌ 错误：无上下文信息
cmd.spawn().map_err(|_| AppError::Io)?;   // 丢失了原始错误类型

// ❌ 错误：固定字符串不传递上下文
.map_err(|_| AppError::Repo("执行失败".into()))?;  // 没有传递原始错误
```

### `let _ =` 的合理使用

`let _ =` 仅在以下场景允许使用，且必须加注释说明原因：

```rust
// ✅ 允许：清理操作失败不影响主流程
let _ = child.kill();   // 终止子进程，失败不必处理
let _ = child.wait();   // 等待回收，失败不必处理

// ❌ 错误：业务操作不应使用 let _ =
let _ = some_business_operation();  // 可能掩盖逻辑错误
```

### `ok()` / `unwrap_or()` / `unwrap_or_else()` 的合理使用

`ok()` 仅在明确不关心错误值的场景允许，`unwrap_or()` 和 `unwrap_or_else()` 用于有默认值的场景：

```rust
// ✅ 允许：有默认值回退
let dir = app_handle.path().app_log_dir()
    .unwrap_or_else(|_| PathBuf::from("."));

// ✅ 允许：在 setup 阶段，只关心 Option 是否为空不关心错误
let tray_icon = app.default_window_icon()
    .cloned()
    .unwrap_or_else(|| tauri::image::Image::new(&[0u8; 256], 16, 16));
```

## 文件拆分时机

### 按文件类型约定

| 类型 | 行数警戒线 | 达到后如何拆分 |
|------|-----------|---------------|
| `commands/*.rs` | 150 行 | 按业务领域拆文件（如 `commands/repo.rs`、`commands/status.rs`） |
| `svn/*.rs` | 300 行 | 按功能拆分子模块（已按 executor/parser/queue/types 拆分） |
| `tools/*.rs` | 100 行 | 每个外部工具类型一个文件（differ/merger/resolver） |
| `models/*.rs` | 150 行 | 按实体拆分（如 `models/repo.rs`、`models/file.rs`） |
| `logging/*.rs` | 100 行 | 日志写入和轮转分离 |
| `config/*.rs` | 100 行 | 配置加载和存储操作分离 |
| `shell_integration/*.rs` | 100 行 | 每平台一个文件 |
| `lib.rs` | 60 行 | 将 command 注册、plugin 注册、setup 各提取到独立子模块 |
| `mod.rs` | 30 行 | 只做声明和重导出，不做任何业务逻辑 |

### 拆分信号

1. **多个 `pub fn` 且前缀重复**（`svn_status`、`svn_commit`）— 暗示子模块
2. **`use` 导入超过 15 个** — 依赖过重
3. **`#[cfg(test)]` 测试块超过 100 行** — 逻辑已够复杂
4. **连续 3 个以上空行** — 一个文件塞了多个功能

### 拆分方式

```
# 拆分前
svn/executor.rs                            ← 280 行，包含 run_svn + status + log + commit + diff + blame

# 拆分后
svn/
├── mod.rs                                   # 模块入口 + pub use 重导出
├── executor.rs                              # CLI 调用 + 超时 + 取消检测
├── parser.rs                                # XML 输出解析
└── types.rs                                 # 共享数据类型
```

`mod.rs` 模式：
```rust
// svn/mod.rs
mod executor;
mod parser;
pub mod types;

// 只重导出外部需要调用的函数
pub use executor::{status, info, commit, log, diff};
```

## 文件级规范
- `mod.rs` 只做模块声明和 `pub use` 重导出，不写业务代码
- 单元测试写在模块末尾 `#[cfg(test)] mod tests { ... }` 块中
- 对外接口添加 `#[tauri::command]` 注解

## 异步规范

- 所有 Command 使用 `async fn`
- 阻塞操作（如 SVN 子进程）使用 `tokio::task::spawn_blocking` 包裹
- 避免在异步上下文中调用 `std::thread::sleep`，使用 `tokio::time::sleep`

---

# 07-死代码预防（AI 自动化）

> AI 自动执行，无需人工检查。

## 编译器可检测的（AI 读到警告即删）

| 代码类型 | AI 策略 |
|----------|---------|
| 未使用的私有函数/变量 | 删除 |
| 未使用的 `use` 导入 | 删除 |
| 未使用的 `struct` 字段 | 删除 |

## 编译器检测不到的（AI 主动检查）

| 代码类型 | AI 检查方式 |
|----------|-------------|
| `pub` 函数 | 回溯调用链 |
| 已注册的 Tauri Command | 检索前端 `invoke('xxx')` 是否存在 |
| 已注册的 Tauri Plugin | 检索前端是否使用对应 API |
| `svn/` 模块中的 `pub` 函数 | 检索 `commands/` 是否导入 |

## AI 清理流程（删除功能时）

1. 删除入口文件/命令，运行 `cargo build 2>&1 | grep "warning:"`
2. 逐条修复编译器警告，重复直到零警告
3. 对每个 `generate_handler!` 中的命令，检索前端是否存在 `invoke("命令名")`
4. 未找到则删除 command 文件 + `commands/mod.rs` 中对应声明
5. 检查 `svn/` 中是否有不再被任何 command 引用的函数
6. 检查 `Cargo.toml` 是否有不再使用的依赖
7. `cargo build && cargo test` 最终验证

## 编码阶段规则

1. 不注释代码 — 不需要的直接 `rm`
2. 重命名后删除旧的 `pub fn`
3. 能用 `pub(crate)` 就不用 `pub`（减少编译器盲区）
4. 注册命令时同步更新 `generate_handler!`，删除命令时同步移除

---

# 08-测试策略

## 单元测试

- 单元测试写在模块末尾 `#[cfg(test)] mod tests { ... }` 中
- 纯逻辑函数（解析器、参数校验、格式化）必须有单元测试
- 测试文件中使用 `unwrap()` 和 `expect()` 是被允许的

## 集成测试

SVN 集成测试使用临时仓库：

```rust
#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::process::Command;

    fn setup_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        // svnadmin create 创建临时仓库
        Command::new("svnadmin")
            .args(["create", dir.path().to_str().unwrap()])
            .output().unwrap();
        dir
    }

    #[test]
    fn test_svn_info() {
        let repo = setup_test_repo();
        // 执行 svn info 测试
        let output = Command::new("svn")
            .args(["info", repo.path().to_str().unwrap()])
            .output().unwrap();
        assert!(output.status.success());
    }
}
```

## Mock 策略

- Service 层涉及 SVN 调用的函数，使用 trait 抽象后可 mock
- Tauri Command 测试不依赖实际 SVN（通过注入 mock service）

## 测试覆盖范围

| 优先级 | 测试内容 | 方法 |
|--------|---------|------|
| P0 | 核心逻辑：XML 解析、参数校验 | 单元测试 |
| P1 | SVN 交互：status/commit/log | 集成测试（临时仓库）|
| P2 | 完整流程：checkout → edit → commit → log | 集成测试 |
| P3 | 错误场景：网络超时、svn 不可用 | mock 注入 |
