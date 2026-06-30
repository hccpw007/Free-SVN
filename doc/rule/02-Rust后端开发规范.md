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
src-tauri/
├── src/
│   ├── main.rs             # 二进制入口，仅调用 lib::run()
│   ├── lib.rs              # Tauri 应用入口，注册插件和命令
│   ├── commands/           # Tauri Command 定义（前端可调用的接口）
│   │   ├── mod.rs
│   │   ├── repo.rs         #   仓库管理命令
│   │   ├── status.rs       #   文件状态命令
│   │   └── commit.rs       #   提交相关命令
│   ├── models/             # 数据模型（结构体、枚举）
│   │   ├── mod.rs
│   │   ├── repo.rs         #   仓库模型
│   │   └── file.rs         #   文件/变更模型
│   └── services/           # 业务逻辑层（SVN 命令执行、数据处理）
│       ├── mod.rs
│       └── svn.rs          #   SVN 命令行封装
├── capabilities/
│   └── default.json        # 权限声明
├── icons/                  # 应用图标
├── Cargo.toml
├── build.rs
└── tauri.conf.json
```

## 分层职责

```
commands/   → 接口层：参数校验、调用 service、返回结果（薄层，不写业务逻辑）
services/   → 业务层：执行 SVN 命令、解析输出、错误处理（核心逻辑所在）
models/     → 数据层：类型定义、序列化（纯数据结构，无方法）
lib.rs      → 应用入口：注册 plugin 和 command
```

禁止反向依赖：`services/` 不能 import `commands/`。

---

# 02-Tauri Command 规范

## 命令定义

每个 Command 放在独立的 rs 文件中，按业务领域划分。

```rust
// commands/repo.rs
use serde::Deserialize;
use crate::models::error::AppError;
use crate::services::svn;

#[derive(Deserialize)]
pub struct OpenRepoParams {
    pub path: String,
}

#[tauri::command]
pub async fn open_repo(params: OpenRepoParams) -> Result<String, AppError> {
    if params.path.is_empty() {
        return Err(AppError::InvalidInput("仓库路径不能为空".into()));
    }
    svn::info(&params.path).await
}
```

## Command 规范

- **Command 函数必须是异步的**（`async fn`）
- 参数较多时（≥2 个）使用 `#[derive(Deserialize)]` 结构体，避免逐个参数传递
- 返回值统一使用 `Result<T, AppError>`，错误信息通过 `AppError` 的 `Display` 实现返回给前端
- 一个 Command 只做一件事，不做"万能接口"
- Command 内部只做参数校验 + 调用 service，不做业务处理

## 注册命令

```rust
// lib.rs
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::repo::open_repo,
            commands::status::get_status,
            commands::commit::create_commit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

# 03-错误处理规范（thiserror）

使用 `thiserror` 定义 `AppError`，Command 返回 `Result<T, AppError>`，Tauri 自动通过 `Display` 序列化错误给前端。

```rust
// Cargo.toml 依赖
// thiserror = "2"

// models/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    /// SVN 命令执行失败
    #[error("SVN 执行失败: {0}")]
    SvnCommand(String),

    /// 参数校验失败
    #[error("参数错误: {0}")]
    InvalidInput(String),

    /// 文件系统错误
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),

    /// 仓库操作异常
    #[error("仓库异常: {0}")]
    Repo(String),

    /// 超时
    #[error("操作超时: {0}")]
    Timeout(String),
}
```

## 规范

- 自定义 `AppError` 枚举，覆盖所有业务错误场景
- 使用 `thiserror` crate 定义错误，通过 `#[error("...")]` 指定显示格式
- Command 返回 `Result<T, AppError>`，Tauri 自动通过 `Display` 将错误信息发送给前端
- 关键路径必须记录日志：`log::error!("...")` / `log::info!("...")`
- 不需要为 AppError 额外实现 `From` 或 `Into` trait，`thiserror` 自动处理

---

# 04-SVN 操作规范

## SVN 命令封装

统一封装在 `services/svn.rs` 中，**禁止**散布 `Command::new("svn")` 调用。

```rust
// services/svn.rs
use std::process::Command;
use log;
use tokio::time::{timeout, Duration};
use tokio::task::spawn_blocking;
use crate::models::error::AppError;

/// 同步执行 SVN 命令（在 spawn_blocking 中运行）
fn run_svn_sync(args: &[&str]) -> Result<String, AppError> {
    log::info!("execute: svn {}", args.join(" "));

    let output = Command::new("svn")
        .args(args)
        .output()
        .map_err(AppError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::SvnCommand(stderr.to_string()))
    }
}

/// 异步执行 SVN 命令（带超时控制）
async fn run_svn(args: &[&str]) -> Result<String, AppError> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    timeout(Duration::from_secs(60), spawn_blocking(move || {
        run_svn_sync(&args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
    }))
    .await
    .map_err(|_| AppError::Timeout("SVN 命令执行超时（60秒）".into()))?
    .map_err(|e| e)?  // 展开 spawn_blocking 的 JoinError
}

/// 获取仓库信息
pub async fn info(path: &str) -> Result<String, AppError> {
    run_svn(&["info", "--xml", path]).await
}

/// 获取文件状态
pub async fn status(path: &str) -> Result<String, AppError> {
    run_svn(&["status", "--xml", path]).await
}
```

## SVN 操作原则

- 所有 SVN 子进程调用走 `run_svn`，禁止散布 `Command::new("svn")`
- 优先 `--xml` 参数获取结构化输出
- 每次执行必须设置超时
- SVN 输出用 serde 反序列化 XML，禁止字符串正则匹配
- 执行前后必须记录日志

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

# 06-编码规范

## 代码风格

| 规则 | 约束 |
|------|------|
| 缩进 | 4 空格 |
| 命名 | 结构体/枚举 PascalCase，函数/变量 snake_case |
| 不可变性 | 优先使用 `let`，仅在必要时用 `let mut` |
| 错误处理 | 使用 `Result<T, E>`，禁止 `unwrap()` 和 `expect()`（仅测试可用） |
| 可见性 | 优先 `pub(crate)`，不暴露不必要的外部 API |

## 文件拆分时机

### 按文件类型约定

| 类型 | 行数警戒线 | 达到后如何拆分 |
|------|-----------|---------------|
| `commands/*.rs` | 150 行 | 按业务领域拆文件（如 `commands/repo.rs`、`commands/status.rs`） |
| `services/*.rs` | 300 行 | 按功能拆分为多个 service 文件（如 `services/svn.rs` → `services/svn/mod.rs` + `services/svn/status.rs`） |
| `models/*.rs` | 150 行 | 按实体拆分（如 `models/repo.rs`、`models/file.rs`） |
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
services/svn.rs                              ← 280 行，包含 run_svn + status + log + commit + diff + blame

# 拆分后
services/svn/
├── mod.rs                                   # 公共 run_svn 函数 + pub use 重导出
├── status.rs                                # svn status 相关
├── commit.rs                                # svn commit 相关
├── log.rs                                   # svn log 相关
└── diff.rs                                  # svn diff 相关
```

`mod.rs` 模式：
```rust
// services/svn/mod.rs
mod status;
mod commit;
mod log;

// 只重导出外部需要调用的函数
pub use status::get_status;
pub use commit::create_commit;
pub use log::get_log;
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

## AI 清理流程（删除功能时）

1. 删除入口文件/命令，运行 `cargo build 2>&1 | grep "warning:"`
2. 逐条修复编译器警告，重复直到零警告
3. 对每个 `generate_handler!` 中的命令，检索前端是否存在 `invoke("命令名")`
4. 未找到则删除 command 文件 + `mod.rs` 中对应声明
5. 检查 `Cargo.toml` 是否有不再使用的依赖
6. `cargo build && cargo test` 最终验证

## 编码阶段规则

1. 不注释代码 — 不需要的直接 `rm`
2. 重命名后删除旧的 `pub fn`
3. 能用 `pub(crate)` 就不用 `pub`（减少编译器盲区）
4. 注册命令时同步更新 `generate_handler!`，删除命令时同步移除

---

# 08-测试策略

- 单元测试写在模块末尾 `#[cfg(test)] mod tests { ... }` 中
- 纯逻辑函数必须有单元测试
- Service 层涉及 SVN 调用的用 mock
- 集成测试用 `svnadmin create` 临时仓库 + `tempfile::TempDir`
- 测试文件中使用 `unwrap()` 和 `expect()` 是被允许的
