<script setup lang="ts">
/** 操作文件行——进度面板中单行文件的图标/状态/路径展示。 */
import { Check, Loader, XCircle } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Props {
  filePath: string
  status: 'completed' | 'in_progress' | 'pending' | 'cancelled'
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
    case 'cancelled': return t('progress.fileLineCancelled')
  }
})
</script>

<template>
  <div
    class="flex items-center gap-2 px-3 py-0.5 font-mono text-sm transition-colors duration-150"
    :class="{
      'text-green-500': status === 'completed',
      'text-amber-600 dark:text-amber-400 bg-slate-100 dark:bg-slate-700': status === 'in_progress',
      'text-slate-100': status === 'pending',
      'text-slate-500': status === 'cancelled',
    }"
    :title="filePath"
    :aria-label="`${statusLabel}: ${filePath}`"
  >
    <!-- 已完成：绿色打钩 -->
    <Check v-if="status === 'completed'" class="shrink-0 w-4 h-4 text-green-500" aria-hidden="true" />
    <!-- 已取消：灰色 X -->
    <XCircle v-else-if="status === 'cancelled'" class="shrink-0 w-4 h-4 text-slate-500" aria-hidden="true" />
    <!-- 正在下载 / 待传输：转圈圈动画 -->
    <Loader v-else class="shrink-0 w-4 h-4 animate-spin" :class="status === 'in_progress' ? 'text-amber-600 dark:text-amber-400' : 'text-slate-400 dark:text-slate-500'" aria-hidden="true" />
    <!-- 文件路径 -->
    <span class="truncate">{{ truncatedPath }}</span>
  </div>
</template>
