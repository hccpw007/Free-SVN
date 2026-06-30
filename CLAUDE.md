# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 最高优先级规则

每次对话结束前，执行 `git add -A && git commit -m "本次修改的中文总结"`，无需询问直接执行。

## 项目概述

Free-SVN — 基于 Tauri + Vue 的跨平台 SVN 图形化工具。

- **前端**：Vue 3 + TypeScript
- **后端**：Rust (Tauri)
- **目标**：macOS / Windows / Linux 三端支持

## 前端样式规范

仅使用 Tailwind CSS 编写样式，**禁止**写 `<style>` 标签或任何自定义 CSS 文件。

## 编码准则（Karpathy）

1. **先想清楚再编码** — 主动暴露假设和不确定性，不做默认选择
2. **只写最简代码** — 不引入未请求的特性或抽象层
3. **外科手术式修改** — 不改动不相关的代码
4. **可验证的成功标准** — 定义目标并循环直到确认完成

## 文档规范

所有文档存放在 `doc/` 目录下，按类别分目录、以序号开头：

| 目录 | 用途 | 命名示例 |
|------|------|----------|
| `doc/design/` | 项目设计文档 | `01-架构设计.md`、`02-数据库设计/` |
| `doc/plan/` | 开发计划 | `01-开发计划.md`、`02-迭代计划/` |
| `doc/rule/` | 规范、规则、代码解释 | `01-开发规范.md`、`02-代码解释/` |
| `doc/report/` | 评估报告、审查报告 | `01-架构评估报告.md` |

- 所有文档文件名**必须以两位数字序号开头**（如 `01-`、`02-`）
- 当某一主题内容较多时，**创建子文件夹**归类存放（如 `02-数据库设计/schema.md`）
- 文档使用 Markdown 格式

## 开发命令

```bash
# 安装依赖
cnpm install

# 开发模式（热更新）
cnpm run tauri dev

# 构建
cnpm run tauri build

# 前端 lint
cnpm run lint
```
