import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppSettings } from '@/types/settings'
import { DEFAULT_SETTINGS } from '@/types/settings'
import { loadSettings, saveSettings } from '@/services/svn'

/** 设置 Store——应用设置加载/保存，含字段映射文档 */
export const useSettingsStore = defineStore('settings', () => {
  const defaultCheckoutDir = ref(DEFAULT_SETTINGS.defaultCheckoutDir)
  const globalIgnorePattern = ref(DEFAULT_SETTINGS.globalIgnorePattern)
  const diffTool = ref(DEFAULT_SETTINGS.diffTool)
  const mergeTool = ref(DEFAULT_SETTINGS.mergeTool)
  const language = ref(DEFAULT_SETTINGS.language)
  const autoStart = ref(DEFAULT_SETTINGS.autoStart)
  const darkMode = ref(DEFAULT_SETTINGS.darkMode)
  const diffCommandTemplate = ref(DEFAULT_SETTINGS.diffCommandTemplate)
  const mergeCommandTemplate = ref(DEFAULT_SETTINGS.mergeCommandTemplate)
  const fallbackToBuiltin = ref(DEFAULT_SETTINGS.fallbackToBuiltin)

  /**
   * 模板字段格式说明：
   *
   * diffCommandTemplate（差异命令模板）：
   *   自定义外部 diff 工具命令格式，支持占位符：
   *     {path}   — 文件路径
   *     {rev1}   — 旧版本号（revision1）
   *     {rev2}   — 新版本号（revision2）
   *     {file1}  — 旧版本临时文件路径
   *     {file2}  — 新版本临时文件路径
   *   示例值: 'code --diff {file1} {file2}'
   *
   * mergeCommandTemplate（合并命令模板）：
   *   自定义外部 merge 工具命令格式，支持占位符：
   *     {base}   — 基础版本文件路径
   *     {mine}   — 我的版本文件路径
   *     {theirs} — 他人版本文件路径
   *     {output} — 输出文件路径
   *   示例值: 'code --merge {mine} {theirs} {base} {output}'
   */

  async function load() {
    try {
      const s = await loadSettings()
      defaultCheckoutDir.value = s.defaultCheckoutDir
      globalIgnorePattern.value = s.globalIgnorePattern || DEFAULT_SETTINGS.globalIgnorePattern
      diffTool.value = s.diffTool
      mergeTool.value = s.mergeTool
      language.value = s.language
      autoStart.value = s.autoStart
      darkMode.value = s.darkMode
      diffCommandTemplate.value = s.diffCommandTemplate || ''
      mergeCommandTemplate.value = s.mergeCommandTemplate || ''
      fallbackToBuiltin.value = s.fallbackToBuiltin ?? true
    } catch (e: unknown) {
      console.error('[settings store] loadSettings 失败，使用默认值:', e)
    }
  }

  /**
   * 保存设置到后端。
   *
   * 前后端字段对应关系（前端 camelCase → 后端 snake_case 自动映射由 serde(rename_all = "camelCase") 处理）：
   *
   * | 前端字段 (AppSettings)        | 后端字段 (Settings struct)    | 类型      | 说明                       |
   * |-------------------------------|------------------------------|-----------|----------------------------|
   * | defaultCheckoutDir            | default_checkout_dir         | string    | 默认检出目录               |
   * | globalIgnorePattern           | global_ignore_pattern        | string    | 全局忽略模式               |
   * | diffTool                      | diff_tool                    | string    | 外部差异对比工具            |
   * | mergeTool                     | merge_tool                   | string    | 外部合并工具                |
   * | diffCommandTemplate           | diff_command_template        | string    | 差异命令模板                |
   * | mergeCommandTemplate          | merge_command_template       | string    | 合并命令模板                |
   * | fallbackToBuiltin             | fallback_to_builtin          | bool      | 外部工具失败时回退内置工具  |
   * | language                      | language                     | string    | 界面语言                    |
   * | autoStart                     | auto_start                   | bool      | 开机自启动                  |
   * | darkMode                      | dark_mode                    | bool      | 暗色模式                    |
   */
  async function save() {
    await saveSettings({
      defaultCheckoutDir: defaultCheckoutDir.value,
      globalIgnorePattern: globalIgnorePattern.value,
      diffTool: diffTool.value,
      mergeTool: mergeTool.value,
      diffCommandTemplate: diffCommandTemplate.value,
      mergeCommandTemplate: mergeCommandTemplate.value,
      fallbackToBuiltin: fallbackToBuiltin.value,
      language: language.value,
      autoStart: autoStart.value,
      darkMode: darkMode.value,
    })
  }

  return {
    defaultCheckoutDir, globalIgnorePattern,
    diffTool, mergeTool, language, autoStart,
    darkMode,
    diffCommandTemplate, mergeCommandTemplate, fallbackToBuiltin,
    load, save,
  }
})
