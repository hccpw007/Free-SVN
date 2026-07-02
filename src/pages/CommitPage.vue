<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useFileListStore } from '@/stores/fileList'
import { storeToRefs } from 'pinia'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { ArrowLeft } from 'lucide-vue-next'
import { Store } from '@tauri-apps/plugin-store'
import { ElMessage, ElMessageBox } from 'element-plus'
import CommitForm from '@/components/svn/CommitForm.vue'
import type { FileItem } from '@/types/svn'

const router = useRouter()
const { t } = useI18n()
const fileListStore = useFileListStore()
const svnStore = useSvnStore()

// 持久化最近提交日志（上限 20 条）
const RECENT_MSG_KEY = 'recentCommitMessages'
const recentMessages = ref<string[]>([])
async function saveRecentMessage(msg: string) {
  if (!msg.trim()) return
  try {
    const store = await Store.load('settings.json')
    const saved = (await store.get<string[]>(RECENT_MSG_KEY)) || []
    const filtered = saved.filter(m => m !== msg)
    const updated = [msg, ...filtered].slice(0, 20)
    recentMessages.value = updated
    await store.set(RECENT_MSG_KEY, updated)
    await store.save()
  } catch (e: unknown) {
      console.warn('[CommitPage] saveRecentMessage 持久化失败:', e)
    }
}

const commitSelected = ref<Set<string>>(new Set())
const commitMessage = ref('')
const isSubmitting = ref(false)
const submitError = ref('')

onMounted(() => { commitSelected.value = new Set(fileListStore.selectedPaths) })

const { files: commitFiles } = storeToRefs(fileListStore)
const selectedCount = computed(() => commitSelected.value.size)

function toggleFile(path: string) {
  const s = new Set(commitSelected.value)
  s.has(path) ? s.delete(path) : s.add(path)
  commitSelected.value = s
}

async function handleCommit() {
  if (selectedCount.value === 0) return
  if (!commitMessage.value.trim()) {
    try {
      await ElMessageBox.confirm(
        t('file.emptyCommitConfirm'),
        t('common.confirm'),
        { confirmButtonText: t('common.confirm'), cancelButtonText: t('common.cancel'), type: 'warning' }
      )
    } catch { return }
  }
  isSubmitting.value = true
  submitError.value = ''
  try {
    const selectedPaths = Array.from(commitSelected.value)

    // 构建 path → status 映射，无须重新调用 SVN
    const pathStatus = new Map(
      commitFiles.value
        .filter(f => commitSelected.value.has(f.path))
        .map(f => [f.path, f.status])
    )

    // 分离需要预处理的文件
    const unversionedPaths = selectedPaths.filter(p => pathStatus.get(p) === 'unversioned')
    const missingPaths = selectedPaths.filter(p => pathStatus.get(p) === 'missing')

    // 步骤1：对未加入文件自动执行 svn add
    if (unversionedPaths.length > 0) {
      await svnStore.addFiles(unversionedPaths)
    }

    // 步骤2：对缺失文件自动执行 svn delete（无 --keep-local，从版本库删除）
    if (missingPaths.length > 0) {
      await svnStore.deleteFiles({ paths: missingPaths, keepLocal: false })
    }

    // 步骤3：提交所有文件
    const result = await svnStore.commit({
      paths: selectedPaths,
      message: commitMessage.value,
      keepLocks: false,
    })
    fileListStore.selectedPaths = new Set()
    commitSelected.value = new Set()
    saveRecentMessage(commitMessage.value)
    const revision = result.detail?.replace(/[^0-9]/g, '')
    ElMessage.success(t('file.commitSuccess', { rev: revision || '' }))
    router.push('/workspace')
  } catch (e: unknown) {
    const error = e as { errorCode?: string; code?: string }
    const errorCode = error?.errorCode || error?.code || ''
    const errorMap: Record<string, string> = {
      'SVN_EXEC_FAILED': 'error.SVN_EXEC_FAILED',
      'SVN_NOT_WORKING_COPY': 'error.notWorkingCopy',
      'SVN_AUTH_FAILED': 'error.authFailed',
      'SVN_CANCELED': 'error.canceled',
      'SVN_OUT_OF_DATE': 'error.outOfDate',
      'SVN_CONFLICT': 'error.conflictDetected',
    }
    const errorKey = errorMap[errorCode] || 'error.SVN_EXEC_FAILED'
    submitError.value = t(errorKey)
  } finally { isSubmitting.value = false }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 顶部导航 -->
    <div class="h-10 px-4 flex items-center gap-2 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 shrink-0">
      <button class="flex items-center gap-1 text-xs text-slate-500 hover:text-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="router.push('/workspace')">
        <ArrowLeft class="w-4 h-4" />{{ t('common.back') }}
      </button>
    </div>
    <!-- 文件列表 -->
    <div class="flex-1 min-h-0 overflow-auto">
      <el-table :data="commitFiles" size="small" style="width:100%" height="100%" row-key="path" @row-click="(r: FileItem) => toggleFile(r.path)">
        <el-table-column width="40">
          <template #default="{ row }"><el-checkbox :model-value="commitSelected.has(row.path)" @change="() => toggleFile(row.path)" /></template>
        </el-table-column>
        <el-table-column :label="t('file.status')" width="48">
          <template #default="{ row }"><span class="text-xs font-mono">{{ row.status }}</span></template>
        </el-table-column>
        <el-table-column :label="t('file.fileName')" min-width="200">
          <template #default="{ row }"><span class="text-xs font-mono truncate block">{{ row.path }}</span></template>
        </el-table-column>
        <el-table-column :label="t('file.fileSize')" width="80">
          <template #default="{ row }"><span class="text-xs font-mono text-slate-400">{{ row.size || '-' }}</span></template>
        </el-table-column>
      </el-table>
    </div>
    <!-- 提交信息输入 -->
    <CommitForm v-model="commitMessage" />
    <!-- 错误提示 -->
    <div v-if="submitError" class="px-4 py-2 text-xs text-red-600 bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-800">{{ submitError }}</div>
    <!-- 底栏 -->
    <div class="h-12 px-4 flex items-center justify-between shrink-0 border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800">
      <span class="text-xs text-slate-500">{{ t('workspace.selectedCount', { count: selectedCount }) }}</span>
      <div class="flex items-center gap-2">
        <button class="px-4 py-1.5 text-xs rounded-md border border-slate-300 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="router.push('/workspace')">{{ t('common.cancel') }}</button>
        <button class="px-4 py-1.5 text-xs rounded-md font-medium text-white bg-green-500 hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" :disabled="selectedCount === 0 || isSubmitting || !commitMessage.value.trim()" :title="selectedCount === 0 ? t('workspace.selectFiles') : ''" @click="handleCommit">{{ isSubmitting ? t('common.submitting') : t('common.submit') }}</button>
      </div>
    </div>
  </div>
</template>
