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

1. **一个文件包含多个 `pub fn` 且前缀重复** — 如 `svn_status()`、`svn_commit()`、`svn_log()` → 说明前缀"svn"暗示了子模块
2. **`use` 导入过多** — 超过 15 个 `use` 语句说明文件依赖过重，需拆分
3. **`#[cfg(test)]` 测试块超过 100 行** — 说明业务逻辑本身已够复杂，应当拆分
4. **单个文件内出现连续 3 个以上空行** — 常用空行分段，说明一个文件里塞了多个功能

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

# 07-死代码预防

## Rust 编译器的检测能力

| 代码类型 | 编译器行为 | 说明 |
|----------|-----------|------|
| 未使用的 `pub` 函数 | **不报错** | Rust 假设 `pub` 可能被外部 crate 调用 |
| 未使用的私有函数/变量 | `warning: unused` | 默认产生警告 |
| 未使用的 `use` 导入 | `warning: unused import` | 默认产生警告 |
| 未使用的 `struct` 字段 | `warning: field is never read` | 默认产生警告 |
| 未使用的 Tauri Command | **不报错** | Command 通过 `generate_handler!` 宏注册，编译器认为它被 "调用了" |

## 必须人工检查的场景（编译器抓不到）

1. **`generate_handler!` 中的命令列表** — 前端不再调用了，但 Rust 编译器认为它还在用
2. **`pub` 函数** — 即使没有被任何地方调用，编译器也不会报 `dead_code`
3. **Tauri Plugin 注册** — 不再需要的 plugin 忘记移除

## 删除功能的 Checklist

```
1. lib.rs → 从 generate_handler! 中移除对应命令
2. lib.rs → 移除不再需要的 plugin 注册
3. commands/ → 删除对应的 .rs 文件，清理 mod.rs
4. services/ → 删除对应的 .rs 文件，清理 mod.rs
5. models/ → 删除对应的 .rs 文件，清理 mod.rs
6. Cargo.toml → 检查是否有不再使用的依赖
7. cargo build → 确认编译通过无警告
```

检查命令：
```bash
cargo build 2>&1 | grep -E "warning: unused|warning: field"  # 列出所有未使用警告
cargo +nightly udeps                                            # 检测未使用的 Cargo 依赖（需 nightly）
```
