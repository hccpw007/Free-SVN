<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Plus, Pencil, Trash2 } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { wrappedInvoke } from '@/services/svn'
import { listCachedCredentials, saveCredentials, deleteCachedCredential, updateCachedCredentialPassword } from '@/services/svn'
import type { CachedCredential } from '@/types/svn'
import { open } from '@tauri-apps/plugin-dialog'

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

// ── Tab 定义 ──
type TabKey = 'svn' | 'diffMerge' | 'interface' | 'general' | 'accountManagement' | 'about'
const activeTab = ref<TabKey>('svn')

interface TabItem {
  key: TabKey
  label: string
}

const tabs: TabItem[] = [
  { key: 'svn', label: t('settings.svn') },
  { key: 'diffMerge', label: t('settings.diffMerge') },
  { key: 'interface', label: t('settings.interface') },
  { key: 'general', label: t('settings.general') },
  { key: 'accountManagement', label: t('settings.accountManagement') },
  { key: 'about', label: t('settings.about') },
]

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

// ── 账号管理 ──
const cachedCredentials = ref<CachedCredential[]>([])
const isLoadingAccounts = ref(false)
const showAddDialog = ref(false)
const showEditDialog = ref(false)
const editingCredential = ref<CachedCredential | null>(null)

// 添加账号表单
const addForm = reactive({
  url: '',
  username: '',
  password: '',
})
const isAdding = ref(false)

// 修改密码表单
const editForm = reactive({
  username: '',
  newPassword: '',
})
const isEditing = ref(false)

async function loadCachedCredentials() {
  isLoadingAccounts.value = true
  try {
    cachedCredentials.value = await listCachedCredentials()
  } catch (e: unknown) {
    console.warn('[SettingsPage] 加载缓存凭据失败:', e)
  } finally {
    isLoadingAccounts.value = false
  }
}

async function handleAddAccount() {
  if (!addForm.url || !addForm.username || !addForm.password) return
  isAdding.value = true
  try {
    await saveCredentials({
      url: addForm.url,
      username: addForm.username,
      password: addForm.password,
    })
    ElMessage.success(t('settings.accountSaved'))
    showAddDialog.value = false
    addForm.url = ''
    addForm.username = ''
    addForm.password = ''
    await loadCachedCredentials()
  } catch (e: unknown) {
    const detail = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e)
    ElMessage.error(detail)
  } finally {
    isAdding.value = false
  }
}

async function handleDeleteCredential(cred: CachedCredential) {
  try {
    await ElMessageBox.confirm(
      t('settings.confirmDeleteAccount', { username: cred.username }),
      t('common.confirm'),
      { confirmButtonText: t('common.delete'), cancelButtonText: t('common.cancel'), type: 'warning' },
    )
    await deleteCachedCredential(cred.id)
    ElMessage.success(t('settings.accountDeleted'))
    await loadCachedCredentials()
  } catch {
    // 用户取消删除不做处理
  }
}

function openEditDialog(cred: CachedCredential) {
  editingCredential.value = cred
  editForm.username = cred.username
  editForm.newPassword = ''
  showEditDialog.value = true
}

async function handleEditPassword() {
  if (!editingCredential.value || !editForm.newPassword) return
  isEditing.value = true
  try {
    await updateCachedCredentialPassword(editingCredential.value.id, editForm.newPassword)
    ElMessage.success(t('settings.passwordUpdated'))
    showEditDialog.value = false
    editingCredential.value = null
    await loadCachedCredentials()
  } catch (e: unknown) {
    const detail = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e)
    ElMessage.error(detail)
  } finally {
    isEditing.value = false
  }
}

function openAddDialog() {
  addForm.url = ''
  addForm.username = ''
  addForm.password = ''
  showAddDialog.value = true
}

// 进入账号管理 tab 时刷新列表
watch(activeTab, (tab) => {
  if (tab === 'accountManagement') {
    loadCachedCredentials()
  }
})
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

        <!-- SVN 设置 -->
        <div v-show="activeTab === 'svn'" class="max-w-xl space-y-6">
          <div>
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.svn') }}</h3>
            <div class="space-y-4">
              <div>
                <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.defaultCheckoutDir') }}</label>
                <div class="flex gap-2 mt-1">
                  <el-input v-model="form.defaultCheckoutDir" size="default" @input="markChanged" />
                  <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="browseDir">{{ t('dialog.browse') }}</el-button>
                </div>
              </div>
              <div>
                <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.globalIgnorePattern') }}</label>
                <el-input v-model="form.globalIgnorePattern" size="default" class="mt-1" @input="markChanged" />
              </div>
            </div>
          </div>
        </div>

        <!-- 差异/合并 -->
        <div v-show="activeTab === 'diffMerge'" class="max-w-xl space-y-6">
          <div>
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.diffMerge') }}</h3>
            <div class="space-y-4">
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
          </div>
        </div>

        <!-- 界面 -->
        <div v-show="activeTab === 'interface'" class="max-w-xl space-y-6">
          <div>
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.interface') }}</h3>
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.darkMode') }}</span>
                <el-switch v-model="form.darkMode" @change="markChanged" />
              </div>
              <div class="flex items-center justify-between">
                <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.showUnversioned') }}</span>
                <el-switch v-model="form.showUnversioned" @change="markChanged" />
              </div>
              <div>
                <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.language') }}</label>
                <el-select v-model="form.language" size="default" class="!w-full mt-1" @change="markChanged">
                  <el-option :label="t('settings.langSystem')" value="system" />
                  <el-option :label="t('settings.langZhCN')" value="zh-CN" />
                  <el-option :label="t('settings.langEn')" value="en" />
                  <el-option :label="t('settings.langJa') + ' (Coming Soon)'" value="ja" />
                  <el-option :label="t('settings.langKo') + ' (Coming Soon)'" value="ko" />
                </el-select>
              </div>
            </div>
          </div>
        </div>

        <!-- 通用 -->
        <div v-show="activeTab === 'general'" class="max-w-xl space-y-6">
          <div>
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.general') }}</h3>
            <div class="flex items-center justify-between">
              <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.autoStart') }}</span>
              <el-switch v-model="form.autoStart" @change="markChanged" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" />
            </div>
          </div>
        </div>

        <!-- 账号管理 -->
        <div v-show="activeTab === 'accountManagement'" class="max-w-2xl space-y-6">
          <div>
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.accountManagement') }}</h3>
              <el-button size="small" type="primary" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="openAddDialog">
                <Plus class="w-4 h-4 mr-1" />{{ t('settings.addAccount') }}
              </el-button>
            </div>

            <!-- 加载中 -->
            <div v-if="isLoadingAccounts" class="py-12 text-center text-sm text-slate-400 dark:text-slate-500">
              {{ t('common.loading') }}
            </div>

            <!-- 无账号 -->
            <div v-else-if="cachedCredentials.length === 0" class="py-12 text-center text-sm text-slate-400 dark:text-slate-500">
              {{ t('settings.noSavedAccounts') }}
            </div>

            <!-- 账号列表 -->
            <div v-else class="border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden">
              <table class="w-full text-sm">
                <thead>
                  <tr class="bg-slate-50 dark:bg-slate-800/50 text-slate-500 dark:text-slate-400">
                    <th class="text-left px-4 py-2.5 font-medium">{{ t('auth.username') }}</th>
                    <th class="text-left px-4 py-2.5 font-medium">{{ t('settings.accountRealm') }}</th>
                    <th class="text-right px-4 py-2.5 font-medium">{{ t('settings.accountActions') }}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-200 dark:divide-slate-700">
                  <tr v-for="cred in cachedCredentials" :key="cred.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/30">
                    <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{{ cred.username }}</td>
                    <td class="px-4 py-3 text-slate-500 dark:text-slate-400 font-mono text-xs">{{ cred.realm }}</td>
                    <td class="px-4 py-3 text-right">
                      <div class="flex items-center justify-end gap-1">
                        <el-button size="small" text class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="openEditDialog(cred)">
                          <Pencil class="w-4 h-4" />
                        </el-button>
                        <el-button size="small" text type="danger" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleDeleteCredential(cred)">
                          <Trash2 class="w-4 h-4" />
                        </el-button>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- 关于 -->
        <div v-show="activeTab === 'about'" class="max-w-xl space-y-6">
          <div>
            <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.about') }}</h3>
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
            <el-button size="small" class="mt-3 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none">{{ t('settings.exportLogs') }}</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 添加账号弹窗 -->
  <el-dialog
    v-model="showAddDialog"
    :title="t('settings.addAccount')"
    :width="'460px'"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
  >
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('settings.repositoryUrl') }}</label>
        <el-input v-model="addForm.url" placeholder="https://svn.example.com/repo" size="default" />
      </div>
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.username') }}</label>
        <el-input v-model="addForm.username" size="default" />
      </div>
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.password') }}</label>
        <el-input v-model="addForm.password" type="password" show-password size="default" />
      </div>
    </div>
    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="showAddDialog = false">
          {{ t('common.cancel') }}
        </el-button>
        <el-button type="primary" size="default" :loading="isAdding" :disabled="!addForm.url || !addForm.username || !addForm.password" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleAddAccount">
          {{ t('common.save') }}
        </el-button>
      </div>
    </template>
  </el-dialog>

  <!-- 修改密码弹窗 -->
  <el-dialog
    v-model="showEditDialog"
    :title="t('settings.modifyPassword')"
    :width="'400px'"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
  >
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.username') }}</label>
        <el-input v-model="editForm.username" disabled size="default" />
      </div>
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('settings.newPassword') }}</label>
        <el-input v-model="editForm.newPassword" type="password" show-password size="default" />
      </div>
    </div>
    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="showEditDialog = false">
          {{ t('common.cancel') }}
        </el-button>
        <el-button type="primary" size="default" :loading="isEditing" :disabled="!editForm.newPassword" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleEditPassword">
          {{ t('common.confirm') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>
