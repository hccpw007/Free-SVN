<script setup lang="ts">
import { ref, computed } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const router = useRouter()
const fileListStore = useFileListStore()
const svnStore = useSvnStore()

const emit = defineEmits<{ close: [] }>()

const conflictFiles = computed(() =>
  fileListStore.files.filter(f => f.status === 'conflicted')
)
const selectedFile = ref(conflictFiles.value[0]?.path || '')
const isResolving = ref(false)

async function handleResolve(method: 'mine-full' | 'theirs-full' | 'working') {
  if (!selectedFile.value) return
  isResolving.value = true
  try {
    await svnStore.resolveConflict({ path: selectedFile.value, resolution: method })
    await fileListStore.refresh()
    emit('close')
  } finally {
    isResolving.value = false
  }
}

async function handleBatchResolve(method: 'mine-full' | 'theirs-full') {
  if (conflictFiles.value.length === 0) return
  isResolving.value = true
  try {
    await Promise.all(
      conflictFiles.value.map(f => svnStore.resolveConflict({ path: f.path, resolution: method }))
    )
    await fileListStore.refresh()
    emit('close')
  } finally {
    isResolving.value = false
  }
}

function viewDiff(filePath: string) {
  if (!filePath) return
  router.push(`/workspace/diff?file=${encodeURIComponent(filePath)}`)
}
</script>

<template>
  <el-dialog :model-value="true" :title="t('dialog.resolveConflict')" width="460px"
    :close-on-click-modal="false" @close="emit('close')">
    <div class="text-sm mb-3">
      <span class="text-slate-400 dark:text-slate-500 text-xs">{{ t('file.conflictFile') }}:</span>
      <el-select v-model="selectedFile" size="small" class="!w-72 ml-2">
        <el-option v-for="f in conflictFiles" :key="f.path" :label="f.path" :value="f.path" />
      </el-select>
    </div>
    <p class="text-xs text-slate-500 dark:text-slate-400 mb-4">
      {{ t('dialog.conflictType') }}: text conflict
    </p>
    <div class="flex gap-2 mb-4">
      <el-button size="small" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="viewDiff(selectedFile)">
        {{ t('dialog.viewDiff') }}
      </el-button>
      <el-button size="small" type="primary" :loading="isResolving"
        class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleResolve('mine-full')">
        {{ t('dialog.useMine') }}
      </el-button>
      <el-button size="small" type="primary" :loading="isResolving"
        class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleResolve('theirs-full')">
        {{ t('dialog.useTheirs') }}
      </el-button>
    </div>
    <div class="flex gap-2 mb-4">
      <el-button size="small" type="primary" :loading="isResolving"
        class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleBatchResolve('mine-full')">
        {{ t('dialog.useMineAll') }}
      </el-button>
      <el-button size="small" type="primary" :loading="isResolving"
        class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleBatchResolve('theirs-full')">
        {{ t('dialog.useTheirsAll') }}
      </el-button>
    </div>
    <p class="text-xs text-slate-400 dark:text-slate-500 mb-4">
      {{ t('dialog.resolveHint') }}
    </p>
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('dialog.cancel') }}</el-button>
      <el-button size="default" type="danger" :loading="isResolving"
        class="focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="handleResolve('working')">
        {{ t('dialog.markResolved') }}
      </el-button>
    </template>
  </el-dialog>
</template>
