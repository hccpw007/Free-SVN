<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const svnStore = useSvnStore()

const emit = defineEmits<{ close: [] }>()

const branches = ref<string[]>([])
const targetUrl = ref('')
const ignoreAncestry = ref(false)
const isSwitching = ref(false)

onMounted(async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    branches.value = await invoke<string[]>('list_branches', { url: workspaceStore.url })
  } catch { /* 忽略：用户仍可手动输入分支路径 */ }
})

async function handleSwitch() {
  if (!targetUrl.value || !workspaceStore.currentPath) return
  isSwitching.value = true
  try {
    await svnStore.switchBranch({
      path: workspaceStore.currentPath,
      targetUrl: targetUrl.value,
      ignoreAncestry: ignoreAncestry.value,
    })
    ElMessage.success(t('workspace.switchSuccess'))
    emit('close')
  } catch { ElMessage.error(t('workspace.switchFailed')) }
  finally { isSwitching.value = false }
}
</script>

<template>
  <el-dialog :model-value="true" :title="t('workspace.switchTitle')" width="460px"
    :close-on-click-modal="false" @close="emit('close')">
    <div class="space-y-3">
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('workspace.switchCurrentPath') }}</label>
        <p class="text-xs font-mono text-slate-700 dark:text-slate-300 mt-1">{{ workspaceStore.currentPath }}</p>
      </div>
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('workspace.switchTarget') }}</label>
        <el-select v-model="targetUrl" filterable allow-create size="default" class="!w-full mt-1" :placeholder="t('workspace.switchTargetPlaceholder')">
          <el-option v-for="b in branches" :key="b" :label="b" :value="b" />
          <template #empty>
            <span class="text-xs text-slate-400 px-2 py-1">{{ t('workspace.noBranchesHint') }}</span>
          </template>
        </el-select>
      </div>
      <el-checkbox v-model="ignoreAncestry"><span class="text-xs">{{ t('workspace.ignoreAncestry') }}</span></el-checkbox>
    </div>
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('common.cancel') }}</el-button>
      <el-button size="default" type="primary" :loading="isSwitching" :disabled="!targetUrl" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleSwitch">{{ t('workspace.switch') }}</el-button>
    </template>
  </el-dialog>
</template>
