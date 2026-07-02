<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { ArrowLeft } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const { t, locale } = useI18n()
const settingsStore = useSettingsStore()

const svnVersion = ref('')
const repoUrl = ref('')

onMounted(async () => {
  try {
    svnVersion.value = await invoke<string>('get_svn_version')
  } catch {
    svnVersion.value = t('common.unknown')
  }
  const ws = useWorkspaceStore()
  repoUrl.value = ws.url || ''
})

watch(
  () => ({
    defaultCheckoutDir: settingsStore.defaultCheckoutDir,
    globalIgnorePattern: settingsStore.globalIgnorePattern,
    darkMode: settingsStore.darkMode,
  }),
  (v) => {
    form.defaultCheckoutDir = v.defaultCheckoutDir
    form.globalIgnorePattern = v.globalIgnorePattern
    form.darkMode = v.darkMode
  },
)

const form = reactive({
  defaultCheckoutDir: settingsStore.defaultCheckoutDir,
  globalIgnorePattern: settingsStore.globalIgnorePattern,
  diffTool: settingsStore.diffTool,
  mergeTool: settingsStore.mergeTool,
  diffCommandTemplate: settingsStore.diffCommandTemplate || '',
  mergeCommandTemplate: settingsStore.mergeCommandTemplate || '',
  fallbackToBuiltin: settingsStore.fallbackToBuiltin,
  showUnversioned: settingsStore.showUnversioned,
  language: settingsStore.language,
  autoStart: settingsStore.autoStart,
  darkMode: settingsStore.darkMode,
})
const hasChanges = ref(false)

function markChanged() { hasChanges.value = true }

async function browseDir() {
  const selected = await open({ directory: true, title: t('dialog.selectTargetDir') })
  if (selected && typeof selected === 'string') {
    form.defaultCheckoutDir = selected
    markChanged()
  }
}

const showDiffCustom = computed(() => form.diffTool === 'custom')
const showMergeCustom = computed(() => form.mergeTool === 'custom')

const validationMessages = computed(() => {
  const msgs: string[] = []
  if (form.defaultCheckoutDir && !/^\/|^[A-Z]:\\/.test(form.defaultCheckoutDir))
    msgs.push(t('settings.checkoutDirInvalidFormat'))
  return msgs
})

async function handleSave() {
  settingsStore.defaultCheckoutDir = form.defaultCheckoutDir
  settingsStore.globalIgnorePattern = form.globalIgnorePattern
  settingsStore.diffTool = form.diffTool
  settingsStore.mergeTool = form.mergeTool
  settingsStore.diffCommandTemplate = form.diffCommandTemplate
  settingsStore.mergeCommandTemplate = form.mergeCommandTemplate
  settingsStore.fallbackToBuiltin = form.fallbackToBuiltin
  settingsStore.showUnversioned = form.showUnversioned
  settingsStore.language = form.language
  settingsStore.autoStart = form.autoStart
  settingsStore.darkMode = form.darkMode
  await settingsStore.save()

  try {
    await invoke('set_auto_start', { enabled: form.autoStart })
  } catch (e) {
    console.warn('Failed to set auto start:', e)
  }

  const langMap: Record<string, string> = {
    system: navigator.language.split('-')[0],
    'zh-CN': 'zh-CN', en: 'en', ja: 'ja', ko: 'ko',
  }
  locale.value = langMap[form.language] || 'en'
  hasChanges.value = false
  ElMessage.success(t('common.saved'))
}
</script>

<template>
  <div class="h-screen flex flex-col bg-slate-50 dark:bg-slate-900">
    <!-- 顶部导航条 -->
    <div class="h-12 px-4 flex items-center gap-2 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 shrink-0">
      <button class="flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition-colors focus:ring-2 focus:ring-blue-400 focus:outline-none rounded"
        @click="router.push('/workspace')">
        <ArrowLeft class="w-4 h-4" />{{ t('common.back') }}
      </button>
      <span class="text-sm font-medium text-slate-800 dark:text-slate-200 ml-2">
        {{ t('settings.title') }}
      </span>
    </div>

    <div class="flex-1 overflow-auto p-6 max-w-2xl mx-auto w-full space-y-4">
      <!-- SVN 设置 -->
      <el-card>
        <template #header><span class="text-sm font-medium">{{ t('settings.svn') }}</span></template>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.defaultCheckoutDir') }}</label>
            <div class="flex gap-2 mt-1">
              <el-input v-model="form.defaultCheckoutDir" size="default" @input="markChanged" />
              <el-button size="default" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="browseDir">{{ t('dialog.browse') }}</el-button>
            </div>
          </div>
          <div>
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.globalIgnorePattern') }}</label>
            <el-input v-model="form.globalIgnorePattern" size="default" class="mt-1" @input="markChanged" />
          </div>
        </div>
      </el-card>

      <!-- 差异/合并 -->
      <el-card>
        <template #header><span class="text-sm font-medium">{{ t('settings.diffMerge') }}</span></template>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffTool') }}</label>
            <el-select v-model="form.diffTool" size="default" class="!w-full mt-1" @change="markChanged">
              <el-option label="内置 (diff2html)" value="builtin" />
              <el-option label="VS Code" value="vscode" />
              <el-option label="Beyond Compare" value="beyond_compare" />
              <el-option label="Kaleidoscope" value="kaleidoscope" />
              <el-option label="自定义" value="custom" />
            </el-select>
          </div>
          <div v-if="showDiffCustom">
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffCommandTemplate') }}</label>
            <el-input v-model="form.diffCommandTemplate" size="default" placeholder="code --diff <file1> <file2>" class="mt-1" @input="markChanged" />
          </div>
          <div>
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeTool') }}</label>
            <el-select v-model="form.mergeTool" size="default" class="!w-full mt-1" @change="markChanged">
              <el-option label="内置 (差异对比+标记)" value="builtin" />
              <el-option label="VS Code" value="vscode" />
              <el-option label="Beyond Compare" value="beyond_compare" />
              <el-option label="Kaleidoscope" value="kaleidoscope" />
              <el-option label="自定义" value="custom" />
            </el-select>
          </div>
          <div v-if="showMergeCustom">
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeCommandTemplate') }}</label>
            <el-input v-model="form.mergeCommandTemplate" size="default" placeholder="bcomp <mine> <base> <theirs> <output>" class="mt-1" @input="markChanged" />
          </div>
          <el-checkbox v-model="form.fallbackToBuiltin" @change="markChanged">
            <span class="text-xs">{{ t('settings.fallbackToBuiltin') }}</span>
          </el-checkbox>
        </div>
      </el-card>

      <!-- 界面 -->
      <el-card>
        <template #header><span class="text-sm font-medium">{{ t('settings.interface') }}</span></template>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-600 dark:text-slate-400">{{ t('settings.showUnversioned') }}</span>
            <el-switch v-model="form.showUnversioned" @change="markChanged" />
          </div>
          <div>
            <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.language') }}</label>
            <el-select v-model="form.language" size="default" class="!w-full mt-1" @change="markChanged">
              <el-option :label="t('settings.langSystem')" value="system" />
              <el-option :label="t('settings.langZhCN')" value="zh-CN" />
              <el-option :label="t('settings.langEn')" value="en" />
              <el-option :label="t('settings.langJa')" value="ja" />
              <el-option :label="t('settings.langKo')" value="ko" />
            </el-select>
          </div>
        </div>
      </el-card>

      <!-- 通用 -->
      <el-card>
        <template #header><span class="text-sm font-medium">{{ t('settings.general') }}</span></template>
        <div class="flex items-center justify-between">
          <span class="text-xs text-slate-600 dark:text-slate-400">{{ t('settings.autoStart') }}</span>
          <el-switch v-model="form.autoStart" @change="markChanged" class="focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" />
        </div>
      </el-card>

      <!-- 关于 -->
      <el-card>
        <template #header><span class="text-sm font-medium">{{ t('settings.about') }}</span></template>
        <div class="space-y-2 text-xs text-slate-400 dark:text-slate-500">
          <p>Free-SVN 版本 0.0.1 · 基于 Tauri 2 · Apache 2.0 许可证</p>
          <p v-if="svnVersion">{{ t('settings.svnVersion') }}: {{ svnVersion }}</p>
          <p v-if="repoUrl">{{ t('settings.repositoryUrl') }}: <span class="font-mono">{{ repoUrl }}</span></p>

          <div class="mt-3 pt-3 border-t border-slate-200 dark:border-slate-700">
            <p class="text-xs font-medium text-slate-500 dark:text-slate-400 mb-2">{{ t('settings.securityTips') }}</p>
            <ul class="space-y-1 ml-4 list-disc">
              <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipPlaintext') }}</li>
              <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipClearAdvice') }}</li>
              <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipSessionOnly') }}</li>
            </ul>
          </div>

          <p class="text-amber-500 dark:text-amber-400 mt-2">{{ t('settings.securityNotice') }}</p>
        </div>
        <el-button size="small" class="mt-3 focus:ring-2 focus:ring-blue-400 focus:outline-none">{{ t('settings.exportLogs') }}</el-button>
      </el-card>

      <!-- 校验提示 -->
      <div v-if="validationMessages.length > 0" class="space-y-1">
        <p v-for="msg in validationMessages" :key="msg" class="text-xs text-amber-600 dark:text-amber-400">⚠️ {{ msg }}</p>
      </div>

      <!-- 保存按钮 -->
      <div class="flex justify-end pb-6">
        <el-button type="primary" :disabled="!hasChanges" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleSave">
          {{ t('common.save') }}
        </el-button>
      </div>
    </div>
  </div>
</template>
