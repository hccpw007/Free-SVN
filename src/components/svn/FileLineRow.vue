<script setup lang="ts">
/** 操作文件行——进度面板中单行文件的图标/状态/路径展示。 */
import { Check, Loader, MoveRight } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Props {
  filePath: string
  status: 'completed' | 'in_progress' | 'pending'
}

const props = withDefaults(defineProps<Props>(), {
  status: 'pending',
})

// 路径截断：前 20 字符 + '...' + 后 15 字符
const truncatedPath = computed(() => {
  if (props.filePath.length <= 38) return props.filePath
  return props.filePath.slice(0, 20) + '...' + props.filePath.slice(-15)
})

// 状态标签（无障碍用）
const statusLabel = computed(() => {
  switch (props.status) {
    case 'completed': return t('progress.fileLineCompleted')
    case 'in_progress': return t('progress.fileLineInProgress')
    case 'pending': return t('progress.fileLinePending')
  }
})
</script>

<template>
  <div
    class="flex items-center gap-2 px-3 py-0.5 font-mono text-sm transition-colors duration-150"
    :class="{
      'text-slate-600 dark:text-slate-600 line-through': status === 'completed',
      'text-amber-600 dark:text-amber-400 bg-slate-100 dark:bg-slate-700': status === 'in_progress',
      'text-slate-600 dark:text-slate-300': status === 'pending',
    }"
    :title="filePath"
    :aria-label="`${statusLabel}: ${filePath}`"
  >
    <!-- 已完成图标 -->
    <Check v-if="status === 'completed'" class="shrink-0 w-4 h-4 text-slate-400 dark:text-slate-600" aria-hidden="true" />
    <!-- 进行中图标（旋转动画） -->
    <Loader v-else-if="status === 'in_progress'" class="shrink-0 w-4 h-4 text-amber-600 dark:text-amber-400 animate-spin" aria-hidden="true" />
    <!-- 待传输图标 -->
    <MoveRight v-else class="shrink-0 w-4 h-4 text-slate-500 dark:text-slate-400" aria-hidden="true" />
    <!-- 文件路径 -->
    <span class="truncate">{{ truncatedPath }}</span>
  </div>
</template>
