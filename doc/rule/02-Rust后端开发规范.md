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
use crate::services::svn;

#[derive(Deserialize)]
pub struct OpenRepoParams {
    pub path: String,
}

#[tauri::command]
pub async fn open_repo(params: OpenRepoParams) -> Result<String, String> {
    if params.path.is_empty() {
        return Err("仓库路径不能为空".into());
    }
    svn::info(&params.path).await
}
```

## Command 规范

- **Command 函数必须是异步的**（`async fn`）
- 参数较多时（≥2 个）使用 `#[derive(Deserialize)]` 结构体，避免逐个参数传递
- 返回值统一使用 `Result<T, String>`，错误信息用中文返回以便前端直接展示
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

# 03-错误处理规范

## 错误类型

```rust
// models/error.rs
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    /// SVN 命令执行失败
    SvnCommand(String),
    /// 参数校验失败
    InvalidInput(String),
    /// 文件系统错误
    Io(std::io::Error),
    /// 仓库操作异常
    Repo(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::SvnCommand(msg) => write!(f, "SVN 执行失败: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "参数错误: {}", msg),
            AppError::Io(e) => write!(f, "文件系统错误: {}", e),
            AppError::Repo(msg) => write!(f, "仓库异常: {}", msg),
        }
    }
}
```

## 错误转换为前端错误

```rust
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
```

## 规范

- 自定义 `AppError` 枚举，覆盖所有业务错误场景
- 使用 `thiserror` crate 简化错误定义（推荐）
- 关键路径必须记录日志：`log::error!("...")` / `log::info!("...")`

---

# 04-SVN 操作规范

## SVN 命令封装

SVN 命令应在 `services/svn.rs` 中统一封装，上层不直接执行子进程。

```rust
// services/svn.rs
use std::process::Command;
use log;
use crate::models::error::AppError;

/// 执行 SVN 命令并返回 stdout
fn run_svn(args: &[&str]) -> Result<String, AppError> {
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

/// 获取仓库信息
pub fn info(path: &str) -> Result<String, AppError> {
    run_svn(&["info", "--xml", path])
}

/// 获取文件状态
pub fn status(path: &str) -> Result<String, AppError> {
    run_svn(&["status", "--xml", path])
}
```

## SVN 操作原则

- 所有 SVN 子进程调用统一通过 `run_svn` 函数，禁止散布 `Command::new("svn")` 调用
- 优先使用 `--xml` 参数获取结构化输出，避免文本解析
- 每次执行设置超时控制
- SVN 输出解析使用 serde 反序列化 XML，避免字符串正则匹配
- 所有 SVN 命令执行前后必须记录日志

---

# 05-安全规范

## 路径安全

```rust
// 禁止直接拼接用户输入
// ❌ 虽然 Rust Command 不经过 shell，但仍需校验路径合法性

fn validate_path(path: &str) -> Result<&str, AppError> {
    if path.contains('\0') {
        return Err(AppError::InvalidInput("路径包含非法字符".into()));
    }
    Ok(path)
}
```

## 命令注入防护

- Rust 的 `Command::new("svn").args(...)` **不经过 shell**，天然防注入
- 禁止使用 `std::process::Command::new("sh").arg("-c")` 执行 SVN 命令

## 敏感信息

- 不在日志中输出用户输入的密码、token 等敏感信息
- `tauri.conf.json` 中不存放任何密钥

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

## 文件级规范

- 每个 `.rs` 文件不超过 300 行
- `mod.rs` 只做模块声明和 `pub use` 重导出，不写业务代码
- 单元测试写在模块末尾 `#[cfg(test)] mod tests { ... }` 块中
- 对外接口添加 `#[tauri::command]` 注解

## 异步规范

- 所有 Command 使用 `async fn`
- 阻塞操作（如 SVN 子进程）使用 `tokio::task::spawn_blocking` 包裹
- 避免在异步上下文中调用 `std::thread::sleep`，使用 `tokio::time::sleep`
