<script setup lang="ts">
/** 创建分支/标签对话框——指定源/目标 URL 和提交信息。 */
import { ref, computed } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const svnStore = useSvnStore()

const emit = defineEmits<{ close: [] }>()

const srcUrl = computed(() => workspaceStore.sourceUrl || workspaceStore.currentPath)
const targetUrl = ref('')
const logMessage = ref('')
const revision = ref<number | undefined>(undefined)
const isCreating = ref(false)

const revisionError = computed(() => {
  if (revision.value !== undefined && revision.value < 1) {
    return t('workspace.revisionRangeError')
  }
  return ''
})

async function handleCreate() {
  if (!targetUrl.value) return
  isCreating.value = true
  try {
    await svnStore.copyBranchTag({
      srcUrl: srcUrl.value,
      dstUrl: targetUrl.value,
      message: logMessage.value || t('workspace.branchDefaultMessage'),
      revision: revision.value,
    })
    ElMessage.success(t('workspace.branchCreated'))
    emit('close')
  } catch (e: unknown) {
    console.error('[BranchTagDialog] 创建分支/标签失败:', e)
    ElMessage.error(t('workspace.branchFailed'))
  }
  finally { isCreating.value = false }
}
</script>

<template>
  <!-- 创建分支/标签对话框 -->
  <el-dialog :model-value="true" :title="t('workspace.branchTagTitle')" width="460px"
    :close-on-click-modal="false" @close="emit('close')">
    <!-- 表单区域 -->
    <div class="space-y-3">
      <!-- 源 URL -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('workspace.sourceUrl') }}</label>
        <p class="text-xs font-mono text-slate-700 dark:text-slate-300 mt-1">{{ srcUrl }}</p>
      </div>
      <!-- 目标 URL -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('workspace.targetUrl') }}</label>
        <el-input v-model="targetUrl" size="default" class="mt-1" :placeholder="t('workspace.targetUrlHint')" />
      </div>
      <!-- 日志信息 -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('common.logMessage') }}</label>
        <el-input v-model="logMessage" type="textarea" :rows="3" size="default" class="mt-1" />
      </div>
      <!-- 版本号 -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('common.revision') }}</label>
        <el-input-number v-model="revision" :min="1" size="default" class="!w-32 mt-1" :placeholder="t('common.optional')" />
        <p v-if="revisionError" class="text-xs text-red-500 mt-1">{{ revisionError }}</p>
      </div>
    </div>
    <!-- 底部按钮 -->
    <template #footer>
      <!-- 关闭对话框 -->
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('common.cancel') }}</el-button>
      <!-- 创建分支/标签 -->
      <el-button size="default" type="primary" :loading="isCreating" :disabled="!targetUrl" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleCreate">{{ t('common.create') }}</el-button>
    </template>
  </el-dialog>
</template>
