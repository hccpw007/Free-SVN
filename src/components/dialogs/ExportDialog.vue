<script setup lang="ts">
/** 导出对话框——导出干净副本到指定目录。 */
import { ref, computed } from 'vue'
import { useSvnStore } from '@/stores/svn'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'

const { t } = useI18n()
const svnStore = useSvnStore()
const workspaceStore = useWorkspaceStore()

const emit = defineEmits<{ close: [] }>()

// 表单状态
const targetDir = ref('')
const revision = ref<number | undefined>(undefined)
const ignoreExternals = ref(false)
const isExporting = ref(false)

const canExport = computed(() => !!workspaceStore.currentPath && !!targetDir.value)

async function browseTarget() {
  const selected = await open({ directory: true, title: t('dialog.selectExportDir') })
  if (selected) targetDir.value = selected
}

async function handleExport() {
  if (!canExport.value) return
  isExporting.value = true
  try {
    await svnStore.exportWorkspace({
      path: workspaceStore.currentPath,
      targetDir: targetDir.value,
      revision: revision.value,
      ignoreExternals: ignoreExternals.value,
    })
    emit('close')
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error('[ExportDialog] 导出失败:', msg)
  } finally {
    isExporting.value = false
  }
}
</script>

<template>
  <el-dialog :model-value="true" :title="t('dialog.exportDialog')" width="480px"
    :close-on-click-modal="false" @close="emit('close')">
    <div class="flex flex-col gap-4">
      <!-- 工作副本路径 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.workspacePath') }}</label>
        <el-input :model-value="workspaceStore.currentPath" size="small" disabled />
      </div>

      <!-- 导出目标路径 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.exportTargetPath') }}</label>
        <div class="flex gap-2">
          <el-input v-model="targetDir" size="small" :placeholder="t('dialog.selectExportDir')" />
          <el-button size="small" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="browseTarget">{{ t('dialog.browse') }}</el-button>
        </div>
      </div>

      <!-- 版本号 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('common.revision') }}</label>
        <el-input-number v-model="revision" :min="1" size="small"
          :placeholder="t('common.optional')" class="!w-32" controls-position="right" />
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ t('dialog.revisionOptional') }}</p>
      </div>

      <el-checkbox v-model="ignoreExternals" size="small">{{ t('dialog.ignoreExternals') }}</el-checkbox>
    </div>

    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')" :disabled="isExporting">
        {{ t('dialog.cancel') }}
      </el-button>
      <el-button size="default" type="primary" :loading="isExporting"
        :disabled="!canExport" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleExport">
        {{ t('dialog.export') }}
      </el-button>
    </template>
  </el-dialog>
</template>
