<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { ArrowLeft } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import { wrappedInvoke } from '@/services/svn'
import SvnSettings from '@/components/settings/SvnSettings.vue'
import IgnoreFilesSettings from '@/components/settings/IgnoreFilesSettings.vue'
import LanguageSettings from '@/components/settings/LanguageSettings.vue'
import GeneralSettings from '@/components/settings/GeneralSettings.vue'
import AccountManagementSettings from '@/components/settings/AccountManagementSettings.vue'
import AboutSettings from '@/components/settings/AboutSettings.vue'

const router = useRouter()
const { t, locale } = useI18n()
const settingsStore = useSettingsStore()

const svnVersion = ref('')
const repoUrl = ref('')

onMounted(async () => {
  try {
    svnVersion.value = await wrappedInvoke<string>('get_svn_version')
  } catch (e: unknown) {
    console.warn('[SettingsPage] 获取 SVN 版本失败:', e)
    svnVersion.value = t('common.unknown')
  }
  const ws = useWorkspaceStore()
  repoUrl.value = ws.url || ''
})

// ── Tab 定义 ──
type TabKey = 'svn' | 'ignoreFiles' | 'language' | 'general' | 'account' | 'about'
const activeTab = ref<TabKey>('svn')

interface TabItem {
  key: TabKey
  label: string
}

const tabs: TabItem[] = [
  { key: 'svn', label: t('settings.svn') },
  { key: 'ignoreFiles', label: t('settings.ignoreFiles') },
  { key: 'language', label: t('settings.languageTab') },
  { key: 'general', label: t('settings.general') },
  { key: 'account', label: t('settings.account') },
  { key: 'about', label: t('settings.about') },
]

// ── 表单 ──
const form = reactive({
  defaultCheckoutDir: settingsStore.defaultCheckoutDir,
  globalIgnorePattern: settingsStore.globalIgnorePattern,
  diffTool: settingsStore.diffTool,
  mergeTool: settingsStore.mergeTool,
  diffCommandTemplate: settingsStore.diffCommandTemplate || '',
  mergeCommandTemplate: settingsStore.mergeCommandTemplate || '',
  fallbackToBuiltin: settingsStore.fallbackToBuiltin,
  language: settingsStore.language,
  autoStart: settingsStore.autoStart,
})
const hasChanges = ref(false)

function markChanged() { hasChanges.value = true }

/** 重置当前 tab 的表单项为默认值 */
function resetTabDefaults() {
  import('@/types/settings').then(({ DEFAULT_SETTINGS }) => {
    switch (activeTab.value) {
      case 'svn':
        form.defaultCheckoutDir = DEFAULT_SETTINGS.defaultCheckoutDir
        form.diffTool = DEFAULT_SETTINGS.diffTool
        form.mergeTool = DEFAULT_SETTINGS.mergeTool
        form.diffCommandTemplate = DEFAULT_SETTINGS.diffCommandTemplate
        form.mergeCommandTemplate = DEFAULT_SETTINGS.mergeCommandTemplate
        form.fallbackToBuiltin = DEFAULT_SETTINGS.fallbackToBuiltin
        break
      case 'ignoreFiles':
        form.globalIgnorePattern = DEFAULT_SETTINGS.globalIgnorePattern
        break
      case 'language':
        form.language = DEFAULT_SETTINGS.language
        break
      case 'general':
        form.autoStart = DEFAULT_SETTINGS.autoStart
        break
    }
    markChanged()
  })
}

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
  settingsStore.language = form.language
  settingsStore.autoStart = form.autoStart
  await settingsStore.save()

  try {
    await wrappedInvoke('set_auto_start', { enabled: form.autoStart })
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
      <button class="flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition-colors focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded"
        @click="router.push('/workspace')">
        <ArrowLeft class="w-4 h-4" />{{ t('common.back') }}
      </button>
      <span class="text-sm font-medium text-slate-800 dark:text-slate-200 ml-2">
        {{ t('settings.title') }}
      </span>
    </div>

    <!-- 左右布局主体 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧竖排 Tab -->
      <div class="w-44 shrink-0 bg-white dark:bg-slate-800 border-r border-slate-200 dark:border-slate-700 flex flex-col py-2">
        <nav class="flex-1">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            class="w-full flex items-center h-10 px-5 text-sm text-left transition-colors border-l-[3px] focus:outline-none focus:ring-1 focus:ring-inset focus:ring-blue-400"
            :class="activeTab === tab.key
              ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 font-medium'
              : 'border-transparent text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700/50 hover:text-slate-800 dark:hover:text-slate-200'"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </nav>

        <!-- 保存按钮固定在左侧底部 -->
        <div class="px-4 pt-2 border-t border-slate-200 dark:border-slate-700">
          <el-button
            type="primary"
            :disabled="!hasChanges"
            class="!w-full focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
            @click="handleSave"
          >
            {{ t('common.save') }}
          </el-button>
        </div>
      </div>

      <!-- 右侧设置内容 -->
      <div class="flex-1 overflow-auto p-8">
        <!-- 校验提示 -->
        <div v-if="validationMessages.length > 0" class="mb-4 space-y-1">
          <p v-for="msg in validationMessages" :key="msg" class="text-xs text-amber-600 dark:text-amber-400">⚠️ {{ msg }}</p>
        </div>

        <div v-show="activeTab === 'svn'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.svn') }}</h3>
            <el-button size="small" @click="resetTabDefaults">{{ t('settings.resetDefaults') }}</el-button>
          </div>
          <SvnSettings
            v-model:defaultCheckoutDir="form.defaultCheckoutDir"
            v-model:diffTool="form.diffTool"
            v-model:diffCommandTemplate="form.diffCommandTemplate"
            v-model:mergeTool="form.mergeTool"
            v-model:mergeCommandTemplate="form.mergeCommandTemplate"
            v-model:fallbackToBuiltin="form.fallbackToBuiltin"
            @changed="markChanged"
          />
        </div>

        <div v-show="activeTab === 'ignoreFiles'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.ignoreFiles') }}</h3>
            <el-button size="small" @click="resetTabDefaults">{{ t('settings.resetDefaults') }}</el-button>
          </div>
          <IgnoreFilesSettings
            v-model:ignorePattern="form.globalIgnorePattern"
            @changed="markChanged"
          />
        </div>

        <div v-show="activeTab === 'language'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.languageTab') }}</h3>
            <el-button size="small" @click="resetTabDefaults">{{ t('settings.resetDefaults') }}</el-button>
          </div>
          <LanguageSettings
            v-model:language="form.language"
            @changed="markChanged"
          />
        </div>

        <div v-show="activeTab === 'general'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.general') }}</h3>
            <el-button size="small" @click="resetTabDefaults">{{ t('settings.resetDefaults') }}</el-button>
          </div>
          <GeneralSettings
            v-model:autoStart="form.autoStart"
            @changed="markChanged"
          />
        </div>

        <div v-show="activeTab === 'account'">
          <AccountManagementSettings />
        </div>

        <div v-show="activeTab === 'about'">
          <AboutSettings :svnVersion="svnVersion" :repoUrl="repoUrl" />
        </div>
      </div>
    </div>
  </div>
</template>
