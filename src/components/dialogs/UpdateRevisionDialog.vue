<script setup lang="ts">
import { ref } from 'vue'
import { useSvnStore } from '@/stores/svn'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'

const { t } = useI18n()
const svnStore = useSvnStore()
const workspaceStore = useWorkspaceStore()

const emit = defineEmits<{ close: [] }>()

// 表单状态
const currentRevision = ref(workspaceStore.currentRevision)
const targetRevision = ref<number | undefined>(undefined)
const updateDepth = ref('') // '' = 不修改
const isUpdating = ref(false)

const SUPPORTED_DEPTHS = [
  { value: '', label: 'dialog.depthUnchanged' },
  { value: 'infinity', label: 'dialog.depthInfinity' },
  { value: 'immediates', label: 'dialog.depthImmediates' },
  { value: 'children', label: 'dialog.depthChildren' },
  { value: 'empty', label: 'dialog.depthEmpty' },
]

async function handleUpdate() {
  if (!workspaceStore.currentPath) return
  isUpdating.value = true
  try {
    await svnStore.updateWorkspace({
      path: workspaceStore.currentPath,
      revision: targetRevision.value,
      depth: updateDepth.value || undefined,
    })
    emit('close')
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    ElMessage.error(msg)
  } finally {
    isUpdating.value = false
  }
}
</script>

<template>
  <el-dialog :model-value="true" :title="t('dialog.updateRevision')" width="420px"
    :close-on-click-modal="false" @close="emit('close')">
    <div class="flex flex-col gap-4">
      <!-- 当前版本 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.currentRevision') }}</label>
        <el-input :model-value="String(currentRevision)" size="small" disabled />
      </div>

      <!-- 目标版本 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.targetRevision') }}</label>
        <el-input-number v-model="targetRevision" :min="1" size="small"
          :placeholder="t('common.optional')" class="!w-32" controls-position="right" />
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ t('dialog.revisionOptional') }}</p>
      </div>

      <!-- 深度 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.depth') }}</label>
        <el-select v-model="updateDepth" size="small" class="!w-full">
          <el-option v-for="opt in SUPPORTED_DEPTHS" :key="opt.value" :value="opt.value" :label="t(opt.label)" />
        </el-select>
      </div>
    </div>

    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')" :disabled="isUpdating">
        {{ t('dialog.cancel') }}
      </el-button>
      <el-button size="default" type="primary" :loading="isUpdating"
        class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleUpdate">
        {{ t('dialog.update') }}
      </el-button>
    </template>
  </el-dialog>
</template>
