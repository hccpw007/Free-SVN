<script setup lang="ts">
/** 忽略规则对话框——文件/扩展名/目录/自定义四种忽略模式。 */
import { ref, computed } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const fileListStore = useFileListStore()
const svnStore = useSvnStore()

const props = defineProps<{ filePath: string }>()
const emit = defineEmits<{ close: [] }>()

const mode = ref<'file' | 'ext' | 'dir' | 'custom'>('file')
const customPattern = ref('')
const isAdding = ref(false)

const computedPattern = computed(() => {
  const fileName = props.filePath.split('/').pop() || ''
  if (mode.value === 'file') return fileName
  if (mode.value === 'ext') {
    const ext = fileName.split('.').pop()
    return ext ? `*.${ext}` : fileName
  }
  if (mode.value === 'dir') return props.filePath.split('/')[0] + '/'
  return customPattern.value
})

async function handleAddIgnore() {
  if (!computedPattern.value) return
  isAdding.value = true
  try {
    await svnStore.setIgnore({ path: props.filePath, pattern: computedPattern.value })
    await fileListStore.refresh()
    emit('close')
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error('[IgnoreDialog] 添加忽略规则失败:', msg)
  } finally {
    isAdding.value = false
  }
}
</script>

<template>
  <el-dialog :model-value="true" :title="t('dialog.ignoreDialog')" width="420px"
    :close-on-click-modal="false" @close="emit('close')">
    <p class="text-xs text-slate-500 dark:text-slate-400 mb-3 font-mono">
      {{ filePath }}
    </p>
    <el-radio-group v-model="mode" class="flex flex-col gap-2 mb-4">
      <el-radio value="file">{{ t('file.ignoreThisFile', { name: props.filePath.split('/').pop() }) }}</el-radio>
      <el-radio value="ext">{{ t('file.ignoreExt', { ext: props.filePath.split('.').pop() }) }}</el-radio>
      <el-radio value="dir">{{ t('file.ignoreDirectory', { dir: props.filePath.split('/')[0] }) }}</el-radio>
      <el-radio value="custom">{{ t('file.ignoreCustom') }}</el-radio>
    </el-radio-group>
    <el-input v-if="mode === 'custom'" v-model="customPattern"
      size="small" class="mb-3" :placeholder="t('file.ignorePlaceholder')" />
    <p class="text-xs text-slate-400 dark:text-slate-500 font-mono bg-slate-50 dark:bg-slate-900 px-2 py-1 rounded">
      svn propset svn:ignore "{{ computedPattern }}" {{ props.filePath }}
    </p>
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')">{{ t('dialog.cancel') }}</el-button>
      <el-button size="default" type="primary" :loading="isAdding"
        :disabled="!computedPattern" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleAddIgnore">
        {{ t('dialog.addIgnore') }}
      </el-button>
    </template>
  </el-dialog>
</template>
