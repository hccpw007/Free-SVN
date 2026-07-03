<script setup lang="ts">
/** 合并向导——四步式合并流程（来源→版本→选项→执行）。 */
import { ref, computed } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const svnStore = useSvnStore()

const emit = defineEmits<{ close: [] }>()

const currentStep = ref(1)
const totalSteps = 4

// 步骤 1: 选择来源
const sourceUrl = ref('')
// 步骤 2: 版本范围
const revStart = ref<number | undefined>(undefined)
const revEnd = ref<number | undefined>(undefined)

const revRangeError = computed(() => {
  if (revStart.value !== undefined && revEnd.value !== undefined && revStart.value > revEnd.value) {
    return t('workspace.mergeRevRangeError')
  }
  return ''
})

// 步骤 3: 合并选项
const ignoreAncestry = ref(false)
const recordOnly = ref(false)
const allowMixedRevisions = ref(false)
// 步骤 4: 执行与结果
const isMerging = ref(false)
type ResultType = 'idle' | 'success' | 'conflict' | 'error'
const resultStatus = ref<ResultType>('idle')
const resultMessage = ref('')

const stepStatus = computed(() => [
  currentStep.value > 1 ? 'finish' : 'process',
  currentStep.value > 2 ? 'finish' : currentStep.value === 2 ? 'process' : 'wait',
  currentStep.value > 3 ? 'finish' : currentStep.value === 3 ? 'process' : 'wait',
  resultStatus.value !== 'idle' ? 'finish' : currentStep.value === 4 ? 'process' : 'wait',
])

function nextStep() {
  if (currentStep.value === 2 && revRangeError.value) return
  if (currentStep.value < totalSteps) currentStep.value++
}
function prevStep() { if (currentStep.value > 1) currentStep.value-- }

async function handleMerge() {
  if (!sourceUrl.value || !workspaceStore.currentPath) return
  isMerging.value = true
  resultStatus.value = 'idle'
  try {
    const r = await svnStore.mergeBranch({
      srcUrl: sourceUrl.value,
      revStart: revStart.value,
      revEnd: revEnd.value,
      targetPath: workspaceStore.currentPath,
    })
    if ('conflictCount' in r && r.conflictCount > 0) {
      resultStatus.value = 'conflict'
      resultMessage.value = t('workspace.mergeConflictResult', { count: r.conflictCount })
    } else {
      resultStatus.value = 'success'
      resultMessage.value = t('workspace.mergeSuccessResult')
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    resultStatus.value = 'error'
    resultMessage.value = msg || t('workspace.mergeFailed')
  }
  finally { isMerging.value = false }
}
</script>

<template>
  <!-- 合并向导弹窗 -->
  <el-dialog :model-value="true" :title="t('workspace.mergeTitle')" width="520px"
    :close-on-click-modal="false" @close="emit('close')">
    <!-- 步骤指示器 -->
    <el-steps :active="currentStep" align-center class="mb-6" :space="120">
      <el-step
        v-for="(stepTitle, i) in [t('workspace.mergeStep1Title'), t('workspace.mergeStep2Title'), t('workspace.mergeStep3Title'), t('workspace.mergeStep4Title')]"
        :key="i" :title="stepTitle" :status="stepStatus[i]" />
    </el-steps>

    <!-- 步骤 1 -->
    <div v-if="currentStep === 1" class="space-y-3">
      <p class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('workspace.mergeStep1') }}</p>
      <el-input v-model="sourceUrl" size="default" :placeholder="t('workspace.sourceUrl')" />
    </div>

    <!-- 步骤 2 -->
    <div v-if="currentStep === 2" class="space-y-3">
      <p class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('workspace.mergeStep2') }}</p>
      <div class="flex gap-3">
        <div>
          <label class="text-xs text-slate-500">{{ t('workspace.revStart') }}</label>
          <el-input-number v-model="revStart" :min="1" size="default" class="!w-32 mt-1" :placeholder="t('common.optional')" />
        </div>
        <div>
          <label class="text-xs text-slate-500">{{ t('workspace.revEnd') }}</label>
          <el-input-number v-model="revEnd" :min="1" size="default" class="!w-32 mt-1" :placeholder="t('common.optional')" />
        </div>
      </div>
      <p v-if="revRangeError" class="text-xs text-red-500">{{ revRangeError }}</p>
    </div>

    <!-- 步骤 3 -->
    <div v-if="currentStep === 3" class="space-y-3">
      <p class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('workspace.mergeStep3') }}</p>
      <div class="flex flex-col gap-2">
        <el-checkbox v-model="ignoreAncestry"><span class="text-xs">{{ t('workspace.ignoreAncestry') }}</span></el-checkbox>
        <el-checkbox v-model="recordOnly"><span class="text-xs">{{ t('workspace.recordOnly') }}</span></el-checkbox>
        <el-checkbox v-model="allowMixedRevisions"><span class="text-xs">{{ t('workspace.allowMixed') }}</span></el-checkbox>
      </div>
    </div>

    <!-- 步骤 4 -->
    <div v-if="currentStep === 4" class="space-y-3">
      <p class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('workspace.mergeStep4') }}</p>
      <div class="bg-slate-50 dark:bg-slate-900 rounded-md p-3 text-xs space-y-1">
        <p class="text-slate-500">{{ t('workspace.mergeTargetLabel') }}: {{ workspaceStore.currentPath }}</p>
        <p class="text-slate-500">{{ t('workspace.mergeSourceLabel') }}: {{ sourceUrl }}</p>
      </div>
      <!-- 执行合并操作 -->
      <el-button size="default" type="primary" :loading="isMerging" :disabled="!sourceUrl"
        class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleMerge">
        {{ t('workspace.executeMerge') }}
      </el-button>
      <!-- 合并结果状态提示 -->
      <div v-if="resultStatus !== 'idle'"
        class="rounded-md p-3 text-xs"
        :class="resultStatus === 'success' ? 'bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-400' : resultStatus === 'conflict' ? 'bg-amber-50 text-amber-700 dark:bg-amber-900/20 dark:text-amber-400' : 'bg-red-50 text-red-700 dark:bg-red-900/20 dark:text-red-400'">
        {{ resultMessage }}
      </div>
      <!-- 查看冲突 -->
      <el-button v-if="resultStatus === 'conflict'" size="small" type="warning" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('dialog.viewConflicts') }}</el-button>
    </div>

    <!-- 底部操作栏 -->
    <template #footer>
      <div class="flex justify-between">
        <!-- 取消关闭对话框 -->
        <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('common.cancel') }}</el-button>
        <!-- 步骤切换按钮组 -->
        <div class="flex gap-2">
          <el-button v-if="currentStep > 1" size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="prevStep">{{ t('common.prev') }}</el-button>
          <el-button v-if="currentStep < totalSteps" size="default" type="primary" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="nextStep">{{ t('common.next') }}</el-button>
          <el-button v-if="currentStep === totalSteps && resultStatus !== 'idle'" size="default" type="primary" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('common.finish') }}</el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>
