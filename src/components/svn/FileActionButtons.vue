<script setup lang="ts">
import { computed } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const props = defineProps<{ file: { status: string; path: string } }>()
const emit = defineEmits<{
  diff: [path: string]
  add: [path: string]
  revert: [path: string]
  ignore: [path: string]
  delete: [path: string]
  merge: [path: string]
  unlock: [path: string]
}>()
const fileListStore = useFileListStore()
const disabled = computed(() => fileListStore.isOperationRunning)
const isOp = (s: string) => props.file.status === s

const btn = (labelKey: string, cls: string, action: string) => ({
  labelKey, cls, handler: () => (emit as any)(action, props.file.path)
})
</script>

<template>
  <div class="flex items-center gap-2">
    <button
      v-for="b in (
        isOp('modified') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.revert','text-amber-600 dark:text-amber-400','revert')] :
        isOp('added') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.ignore','text-slate-500 dark:text-slate-400','ignore'),btn('file.delete','text-red-500 dark:text-red-400','delete')] :
        isOp('conflicted') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.merge','text-red-600 dark:text-red-400','merge')] :
        isOp('deleted') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.ignore','text-slate-500 dark:text-slate-400','ignore'),btn('file.revert','text-amber-600 dark:text-amber-400','revert')] :
        isOp('unversioned') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.add','text-green-600 dark:text-green-400','add'),btn('file.ignore','text-slate-500 dark:text-slate-400','ignore')] :
        isOp('missing') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.add','text-green-600 dark:text-green-400','add'),btn('file.delete','text-red-500 dark:text-red-400','delete')] :
        isOp('locked') ? [btn('file.diff','text-blue-600 dark:text-blue-400','diff'),btn('file.unlock','text-violet-600 dark:text-violet-400','unlock')] : []
      )" :key="b.labelKey" :disabled="disabled"
      class="text-xs underline-offset-2 hover:underline transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
      :class="[b.cls, disabled ? 'opacity-50 cursor-not-allowed' : '']"
      @click.stop="b.handler"
    >{{ t(b.labelKey) }}</button>
  </div>
</template>
