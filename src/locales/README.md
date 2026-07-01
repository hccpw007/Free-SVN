# locale 翻译文件说明

## 当前状态

| 文件 | 语言 | 状态 |
|------|------|------|
| `zh-CN.json` | 简体中文 | ✅ 完整翻译，已对齐前端全部 UI 文案 |
| `en.json` | English | ✅ 完整翻译，已对齐前端全部 UI 文案 |
| `ja.json` | 日本語 | ⚠️ 当前使用英文占位，待后续阶段完善母语翻译 |
| `ko.json` | 한국어 | ⚠️ 当前使用英文占位，待后续阶段完善母语翻译 |

## 回退机制

`vue-i18n` 配置了 `fallbackLocale: 'en'`，因此 ja/ko 中未翻译的 key 将自动回退显示为英文。当前阶段 ja.json 和 ko.json 的完整英文内容确保了未翻译状态的界面可用性。

## 后续完善计划

ja.json 和 ko.json 需在后续阶段由母语译者完善为完整的日文/韩文翻译，届时需：

1. 替换全部 5 个分类（`app`、`error`、`common`、`workspace`、`file`）的文案为对应语言
2. 保持 JSON 结构与 `zh-CN.json`/`en.json` 完全一致
3. 更新 `LANG_MAP`（定义于 `src/main.ts`）中的显示名称（如有需要）
4. 在设置页的语言选项中添加对应语言的标注（如 `日本語 (Coming Soon)` → `日本語`）
