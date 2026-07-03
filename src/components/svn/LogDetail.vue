<script setup lang="ts">
/** 日志详情——展开日志条目展示作者/日期/变更文件列表。 */
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface LogEntry {
  revision: number
  author: string
  date: string
  msg: string
  paths: Array<{ action: string; path: string }>
}

const props = defineProps<{ log: LogEntry }>()
const emit = defineEmits<{ viewDiff: [file: string, revision: number] }>()
const files = computed(() => props.log.paths || [])
</script>

<template>
  <!-- 日志详情区块 -->
  <div class="px-4 py-3 bg-slate-50 dark:bg-slate-800/50 rounded-md">
    <!-- 提交元信息 -->
    <div class="grid grid-cols-3 gap-4 text-xs mb-3">
      <div><span class="text-slate-400">{{ t('log.author') }}:</span> <span class="text-slate-700 dark:text-slate-300">{{ log.author }}</span></div>
      <div><span class="text-slate-400">{{ t('log.date') }}:</span> <span class="text-slate-700 dark:text-slate-300 font-mono">{{ log.date }}</span></div>
      <div><span class="text-slate-400">{{ t('log.revision') }}:</span> <span class="text-slate-700 dark:text-slate-300 font-mono">r{{ log.revision }}</span></div>
    </div>
    <div class="text-xs text-slate-600 dark:text-slate-400 mb-3 whitespace-pre-wrap">{{ log.msg || log.message }}</div>
    <!-- 变更文件列表 -->
    <div>
      <p class="text-xs font-medium text-slate-500 dark:text-slate-400 mb-1">{{ t('log.changedFiles') }}:</p>
      <div :class="files.length > 30 ? 'max-h-40 overflow-y-auto' : ''">
        <!-- 遍历文件列表 -->
        <div v-for="f in files" :key="f.path" class="flex items-center gap-2 py-0.5 hover:bg-slate-100 dark:hover:bg-slate-700/50 rounded px-1 group">
          <span class="w-5 text-xs font-mono text-center shrink-0">{{ f.action }}</span>
          <span class="text-xs font-mono text-slate-600 dark:text-slate-400 truncate flex-1">{{ f.path }}</span>
          <!-- 查看差异 -->
          <button class="text-xs text-blue-600 hover:underline shrink-0 opacity-0 group-hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="emit('viewDiff', f.path, log.revision)">{{ t('log.viewDiff') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
