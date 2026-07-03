# Rust 后端开发规范

## 01-技术栈

| 层面 | 选型 |
|------|------|
| 语言 | Rust (edition 2021) |
| 框架 | Tauri 2.x |
| 序列化 | serde + serde_json |
| 错误处理 | thiserror |
| 异步 | tokio（Tauri 内置） |
| 日志 | tauri-plugin-log + log crate |
| 持久化 | tauri-plugin-store |

## 02-项目结构

```
src-tauri/src/
├── commands/   # Tauri Command（参数校验 + 调用下层，薄层）
├── svn/        # SVN CLI 核心（executor/parser/queue/types）
├── models/     # 数据模型（纯 struct + enum，无方法）
├── services/   # 业务逻辑层（预留）
├── tools/      # 外部差异/合并工具调用
├── logging/    # 日志写入和轮转
├── config/     # 配置持久化（tauri-plugin-store 封装）
├── lib.rs      # 入口：注册 plugin + command
└── main.rs     # 仅调用 lib::run()
```

禁止反向依赖：`svn/` 不能 import `commands/`。

## 03-Tauri Command 规范

- Command 函数必须是 `async fn`
- 参数较多（≥2）时使用 `#[derive(Deserialize)]` 结构体
- 返回值统一 `Result<T, AppError>`
- 一个 Command 只做一件事，不做"万能接口"
- Command 只做参数校验 + 调用业务模块，不做业务处理

```rust
#[derive(Deserialize)]
pub struct StatusParams { pub path: String }

#[tauri::command]
pub async fn get_status(params: StatusParams) -> Result<SvnStatusOutput, AppError> {
    svn::executor::status(&params.path).await
}
```

### 长操作 Tauri Event 协议

| 事件名 | Payload | 触发时机 |
|--------|---------|---------|
| `operation:started` | `{ operation }` | 长操作开始时 |
| `operation:progress` | `{ percent: 0-100, stage, fileCount }` | 进度更新 |
| `operation:completed` | `{ result, detail? }` | 完成/取消/失败 |

前后端类型同步：修改 Command 签名时必须同步更新另一侧类型定义。

## 04-错误处理（thiserror）

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("SVN_EXEC_FAILED: {0}")]  SvnCommand(String),
    #[error("SVN_PARSE_FAILED: {0}")] ParseFailed(String),
    #[error("INVALID_INPUT: {0}")]    InvalidInput(String),
    #[error("IO_ERROR: {0}")]         Io(#[from] std::io::Error),
    #[error("SVN_TIMEOUT: {0}")]      Timeout(String),
    #[error("SVN_NOT_FOUND")]         SvnNotFound,
    #[error("SVN_NOT_WORKING_COPY")]  NotWorkingCopy,
    #[error("SVN_CANCELLED")]         Cancelled,
    #[error("NETWORK_UNREACHABLE")]   NetworkUnreachable,
}
```

**规则：** 后端返回错误码 key（英文大写），前端根据 locale 翻译。日志中记录原始 stderr。

## 05-SVN 操作规范

```
svn/mod.rs       # 模块入口
svn/executor.rs  # CLI 调用 + 超时 60s + 取消检测
svn/parser.rs    # XML 输出解析（serde 反序列化）
svn/queue.rs     # 操作队列与并发控制（Mutex）
svn/types.rs     # 共享数据类型
```

- 所有 SVN 调用走 `svn::executor`，禁止散布 `Command::new("svn")`
- 优先 `--xml` 参数 + serde 解析，禁止字符串正则解析
- 写操作通过 `svn::queue::SvnQueue` 互斥（同一工作副本）
- 长操作循环中检测 `is_cancelled()`，服务端操作前检测网络

## 06-编码约定

| 规则 | 约束 |
|------|------|
| 缩进 | 4 空格 |
| 命名 | struct/enum PascalCase，函数/变量 snake_case |
| 不可变性 | 优先 `let`，仅在必要时用 `let mut` |
| 错误处理 | 使用 `Result<T, E>`，禁止 `unwrap()` / `expect()`（仅测试可用） |
| 可见性 | 优先 `pub(crate)`，不暴露外部 API |
| 异步 | `async fn` + `tokio::task::spawn_blocking` 包裹阻塞操作 |
| 日志 | 每个 map_err 包含可诊断的描述信息 |

### 文件拆分标准

**警戒线以纯代码行数为准，注释行和空行不计入。**

| 类型 | 警戒线 | 拆分方式 |
|------|--------|----------|
| `commands/*.rs` | 150 行 | 按领域拆文件 |
| `svn/*.rs` | 300 行 | 按功能拆子模块 |
| `models/*.rs` | 150 行 | 按实体拆分 |
| `lib.rs` | 60 行 | 提取 plugin/setup/command 到子模块 |

拆分信号：多个 `pub fn` 且前缀重复 / `use` 超过 15 个 / `#[cfg(test)]` 超过 100 行。

## 07-死代码预防

> CLAUDE.md 已有规则，此处仅说明工具：
> - 编译器可检测的：未使用函数/变量/import — 读到警告即删
> - 编译器检测不到的：`pub` 函数回溯调用链，Tauri Command 检索前端 `invoke` 是否存在
> - 清理流程：删入口 → `cargo build` 修警告 → 查 `generate_handler!` → 删残余 → `cargo build + test` 验证
