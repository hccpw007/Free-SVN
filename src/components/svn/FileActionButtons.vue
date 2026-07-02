<script setup lang="ts">
import { computed } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const props = defineProps<{ file: { status: string; path: string } }>()
const emit = defineEmits<{
  diff: [path: string]
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
  labelKey, cls, handler: () => emit(action as any, props.file.path)
})
</script>

<template>
  <div class="flex items-center gap-2">
    <button
      v-for="b in (
        isOp('modified') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionRevert','text-amber-600','revert')] :
        isOp('added') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionIgnore','text-slate-500','ignore'),btn('file.actionDelete','text-red-500','delete')] :
        isOp('conflicted') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionMerge','text-red-600','merge')] :
        isOp('deleted') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionIgnore','text-slate-500','ignore'),btn('file.actionRevert','text-amber-600','revert')] :
        isOp('unversioned') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionIgnore','text-slate-500','ignore')] :
        isOp('locked') ? [btn('file.actionDiff','text-blue-600','diff'),btn('file.actionUnlock','text-violet-600','unlock')] : []
      )" :key="b.labelKey" :disabled="disabled"
      class="text-xs underline-offset-2 hover:underline transition-colors duration-150 focus:ring-2 focus:ring-blue-400 focus:outline-none"
      :class="[b.cls, disabled ? 'opacity-50 cursor-not-allowed' : '']"
      @click.stop="b.handler"
    >{{ t(b.labelKey) }}</button>
  </div>
</template>
