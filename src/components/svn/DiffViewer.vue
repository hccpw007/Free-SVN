<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useFileListStore } from '@/stores/fileList'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { html } from 'diff2html'
import { X, ChevronUp, ChevronDown, Search } from 'lucide-vue-next'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'

const { t } = useI18n()
const fileListStore = useFileListStore()
const svnStore = useSvnStore()
const route = useRoute()

const diffFile = ref<{
  path: string
  content: string
  isBinary: boolean
  rev1?: number
  rev2?: number
} | null>(null)
const isVisible = computed(() => !!diffFile.value)
const panelHeight = ref(Math.min(360, window.innerHeight * 0.4))
const isDragging = ref(false)
const isLoading = ref(false)

// diff2html 渲染
const diffHtml = computed(() => {
  if (!diffFile.value || diffFile.value.isBinary) return ''
  if (!diffFile.value.content) return '<p class="text-sm text-slate-400 p-4">差异内容为空</p>'
  try {
    return html(diffFile.value.content, {
      drawFileList: false,
      matching: 'lines',
      outputFormat: 'line-by-file',
      highlight: true,
    })
  } catch { return '<p class="text-sm text-red-500 p-4">差异渲染失败</p>' }
})

const panelTitle = computed(() => {
  const f = diffFile.value
  if (!f) return ''
  const revInfo = f.rev1 !== undefined ? t('common.revisionRange', { r1: f.rev1, r2: f.rev2 ?? '' }) : ''
  return `${f.path}${revInfo ? ` (${revInfo})` : ''}`
})

async function openDiff(path: string) {
  isLoading.value = true
  try {
    const r = await svnStore.getDiff({ path })
    diffFile.value = { path, content: r.content, isBinary: r.isBinary, rev1: r.revision1, rev2: r.revision2 }
  } catch {
    diffFile.value = { path, content: '', isBinary: false }
  }
  finally { isLoading.value = false }
}

function closeDiff() { diffFile.value = null }

// Ctrl+F 搜索
const searchQuery = ref('')
const searchVisible = ref(false)
function toggleSearch() { searchVisible.value = !searchVisible.value }

const highlightedDiffHtml = computed(() => {
  const raw = diffHtml.value
  if (!raw || !searchQuery.value.trim() || !searchVisible.value) return raw
  const escaped = searchQuery.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return raw.replace(new RegExp(`(${escaped})`, 'gi'),
    '<mark class="bg-yellow-200 dark:bg-yellow-600/40 rounded px-0.5">$1</mark>')
})

function doSearch() {
  if (!searchQuery.value.trim() || !container.value) return
  const firstMark = container.value.querySelector('mark')
  if (firstMark) {
    firstMark.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

// 外部工具打开 / 复制路径
async function openInExternalTool(path: string) {
  try { const { open } = await import('@tauri-apps/plugin-shell'); await open(path) } catch { /* fallback */ }
}
async function copyFilePath(path: string) {
  try { await navigator.clipboard.writeText(path) } catch { /* fallback */ }
}

// 拖拽条
function startDrag(e: MouseEvent) {
  isDragging.value = true
  const sy = e.clientY, sh = panelHeight.value
  const move = (ev: MouseEvent) => {
    panelHeight.value = Math.max(120, Math.min(window.innerHeight * 0.7, sh + sy - ev.clientY))
  }
  const up = () => {
    isDragging.value = false
    window.removeEventListener('mousemove', move)
    window.removeEventListener('mouseup', up)
  }
  window.addEventListener('mousemove', move)
  window.addEventListener('mouseup', up)
}

// 逐块导航
const container = ref<HTMLElement | null>(null)
function navDiff(dir: 'prev' | 'next') {
  const el = container.value
  if (!el) return
  const blocks = el.querySelectorAll('.d2h-ins, .d2h-del')
  if (!blocks.length) return
  const center = el.scrollTop + el.clientHeight / 2
  let idx = -1
  blocks.forEach((b, i) => {
    const top = (b as HTMLElement).offsetTop
    if (dir === 'next' && top > center + 50 && idx === -1) idx = i
    if (dir === 'prev' && top < center - 50) idx = i
  })
  if (idx >= 0) blocks[idx].scrollIntoView({ behavior: 'smooth', block: 'center' })
}

// 键盘快捷键
const { register: regShortcut, unregister: unregShortcut } = useKeyboardShortcuts()

onMounted(() => {
  regShortcut([
    { id: 'diff-close', keys: 'Escape', handler: () => closeDiff(), scope: 'panel' },
    { id: 'diff-close2', keys: 'ctrl+w', handler: () => closeDiff(), scope: 'panel' },
    { id: 'diff-next', keys: 'ctrl+ArrowDown', handler: () => navDiff('next'), scope: 'panel' },
    { id: 'diff-prev', keys: 'ctrl+ArrowUp', handler: () => navDiff('prev'), scope: 'panel' },
    { id: 'diff-search', keys: 'ctrl+f', handler: () => { searchVisible.value = true }, scope: 'panel' },
  ])
})

onUnmounted(() => {
  unregShortcut('diff-close')
  unregShortcut('diff-close2')
  unregShortcut('diff-next')
  unregShortcut('diff-prev')
  unregShortcut('diff-search')
})

// 路由入口
watch(() => route.query.file, (f) => { if (f && typeof f === 'string') openDiff(f) })
</script>

<template>
  <Transition name="diff-slide"
    enter-from-class="translate-y-full opacity-0"
    enter-active-class="transition-all duration-200 ease-in-out"
    leave-active-class="transition-all duration-200 ease-in-out"
    leave-to-class="translate-y-full opacity-0"
  >
    <div v-if="isVisible"
      class="border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 flex flex-col overflow-hidden"
      :style="{ height: panelHeight + 'px' }"
    >
      <!-- 拖拽条 -->
      <div class="h-1 cursor-row-resize shrink-0 hover:bg-green-500/30 relative -mt-0.5 z-10" @mousedown="startDrag" />
      <!-- 标题栏 -->
      <div class="h-8 px-4 flex items-center justify-between shrink-0 bg-slate-50 dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700">
        <span class="text-xs font-mono text-slate-600 dark:text-slate-300 truncate">{{ panelTitle }}</span>
        <div class="flex items-center gap-2">
          <button class="text-xs text-slate-400 hover:text-slate-600 flex items-center gap-0.5 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" :title="t('common.search')" @click="toggleSearch"><Search class="w-3 h-3" /></button>
          <button v-if="diffFile" class="text-xs text-slate-400 hover:text-blue-600 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" :title="t('file.openWithEditor')" @click="openInExternalTool(diffFile.path)">{{ t('file.openWithEditor') }}</button>
          <button v-if="diffFile" class="text-xs text-slate-400 hover:text-blue-600 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" :title="t('file.copyFullPath')" @click="copyFilePath(diffFile.path)">{{ t('file.copyFullPath') }}</button>
          <button class="text-xs text-blue-600 hover:underline flex items-center gap-0.5 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="navDiff('prev')"><ChevronUp class="w-3 h-3" />{{ t('file.diffPrev') }}</button>
          <button class="text-xs text-blue-600 hover:underline flex items-center gap-0.5 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="navDiff('next')">{{ t('file.diffNext') }}<ChevronDown class="w-3 h-3" /></button>
          <button class="p-0.5 rounded text-slate-400 hover:text-slate-600 focus:ring-2 focus:ring-blue-400 focus:outline-none" :title="t('common.close')" @click="closeDiff"><X class="w-4 h-4" /></button>
        </div>
      </div>
      <!-- 搜索输入框 -->
      <div v-if="searchVisible" class="px-4 py-1.5 border-b border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800">
        <el-input v-model="searchQuery" size="small" :placeholder="t('diff.searchPlaceholder')" clearable @input="doSearch" @clear="searchQuery = ''" />
      </div>
      <!-- 内容区 -->
      <div ref="container" class="flex-1 overflow-auto text-xs">
        <div v-if="diffFile?.isBinary" class="flex items-center justify-center h-full">
          <p class="text-sm text-slate-500 dark:text-slate-400">{{ t('file.binaryFileHint') }}</p>
        </div>
        <div v-else-if="isLoading" class="flex items-center justify-center h-full">
          <div class="w-full max-w-2xl px-4 space-y-3">
            <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-3/4" />
            <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-1/2" />
            <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-5/6" />
            <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-2/3" />
            <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded animate-pulse w-4/5" />
          </div>
        </div>
        <div v-else-if="highlightedDiffHtml" v-html="highlightedDiffHtml" class="[&_.d2h-file-wrapper]:border-0 [&_.d2h-file-header]:hidden" />
      </div>
    </div>
  </Transition>
</template>
