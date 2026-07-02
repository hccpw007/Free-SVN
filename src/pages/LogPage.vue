<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSvnStore } from '@/stores/svn'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Search, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import LogDetail from '@/components/svn/LogDetail.vue'
import type { LogEntry } from '@/types/svn'

const router = useRouter()
const { t } = useI18n()
const svnStore = useSvnStore()
const ws = useWorkspaceStore()

const logs = ref<LogEntry[]>([])
const isLoading = ref(false)
const searchQuery = ref('')
const currentPage = ref(1)
const pageSize = 50
const totalLogs = ref(0)
const expandedRow = ref<number | null>(null)
let timer: any

// 搜索防抖 300ms
watch(searchQuery, () => {
  clearTimeout(timer)
  timer = setTimeout(() => { currentPage.value = 1; fetchLogs() }, 300)
})

async function fetchLogs() {
  if (!ws.currentPath) return
  isLoading.value = true
  try {
    const r = await svnStore.getLog({
      path: ws.currentPath,
      limit: pageSize,
      search: searchQuery.value || undefined,
    })
    logs.value = r.entries
    totalLogs.value = r.totalCount
  } catch { logs.value = [] }
  finally { isLoading.value = false }
}

onMounted(() => fetchLogs())

function toggleRow(rev: number) {
  expandedRow.value = expandedRow.value === rev ? null : rev
}
async function copyRev(rev: number) {
  await navigator.clipboard.writeText(String(rev))
}
function prevPage() { if (currentPage.value > 1) { currentPage.value--; fetchLogs() } }
function nextPage() { currentPage.value++; fetchLogs() }
function viewDiff(file: string, rev: number) {
  router.push(`/workspace/diff?file=${encodeURIComponent(file)}&revision=${rev}`)
}
const totalPages = () => Math.ceil(totalLogs.value / pageSize) || 1
const isSearchEmpty = computed(() => logs.value.length === 0 && !!searchQuery.value && !isLoading.value)
const isLogEmpty = computed(() => logs.value.length === 0 && !searchQuery.value && !isLoading.value)
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 顶部导航 -->
    <div class="h-10 px-4 flex items-center gap-2 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 shrink-0">
      <button class="flex items-center gap-1 text-xs text-slate-500 hover:text-slate-700 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="router.push('/workspace')">
        <ArrowLeft class="w-4 h-4" />{{ t('common.back') }}
      </button>
    </div>
    <!-- 搜索栏 -->
    <div class="px-4 py-2 flex items-center gap-3 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 shrink-0">
      <el-input v-model="searchQuery" :placeholder="t('workspace.searchLogPlaceholder')" size="default" clearable class="max-w-sm">
        <template #prefix><Search class="w-4 h-4 text-slate-400" /></template>
      </el-input>
    </div>
    <!-- 日志列表 -->
    <div class="flex-1 min-h-0 overflow-auto">
      <el-table
        :data="logs" size="small" style="width:100%" v-loading="isLoading"
        @row-click="(r: any) => toggleRow(r.revision)"
        :row-class-name="'cursor-pointer'" row-key="revision"
      >
        <el-table-column :label="t('log.revision')" width="80" sortable="custom" prop="revision">
          <template #default="{ row }">
            <span class="text-xs font-mono text-blue-600 dark:text-blue-400 cursor-pointer hover:underline" @click.stop="copyRev(row.revision)">r{{ row.revision }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('log.date')" width="140" prop="date">
          <template #default="{ row }"><span class="text-xs font-mono text-slate-600 dark:text-slate-400">{{ row.date }}</span></template>
        </el-table-column>
        <el-table-column :label="t('log.author')" width="100" prop="author">
          <template #default="{ row }"><span class="text-xs text-slate-600 dark:text-slate-400">{{ row.author }}</span></template>
        </el-table-column>
        <el-table-column :label="t('log.message')" min-width="300" prop="msg">
          <template #default="{ row }"><span class="text-xs text-slate-700 dark:text-slate-300 truncate block">{{ row.msg }}</span></template>
        </el-table-column>
        <!-- 展开列 -->
        <el-table-column type="expand" width="30">
          <template #default="{ row }">
            <Transition enter-active-class="transition-all duration-200 ease-out" leave-active-class="transition-all duration-200 ease-in"
              enter-from-class="opacity-0 max-h-0" enter-to-class="opacity-100 max-h-[500px]"
              leave-from-class="opacity-100 max-h-[500px]" leave-to-class="opacity-0 max-h-0">
              <div v-if="expandedRow === row.revision" :key="row.revision">
                <LogDetail :log="row" @view-diff="viewDiff" />
              </div>
            </Transition>
          </template>
        </el-table-column>
      </el-table>
      <!-- 搜索无匹配结果 -->
      <div v-if="isSearchEmpty" class="flex items-center justify-center h-full">
        <div class="text-center">
          <Search class="w-8 h-8 mx-auto text-slate-300 dark:text-slate-600" />
          <p class="mt-2 text-sm text-slate-500 dark:text-slate-400">{{ t('workspace.logNoSearchResult') }}</p>
          <p class="mt-1 text-xs text-slate-400 dark:text-slate-500">{{ t('workspace.searchFilterHint') }}</p>
        </div>
      </div>
      <!-- 日志为空 -->
      <div v-if="isLogEmpty" class="flex items-center justify-center h-full">
        <p class="text-sm text-slate-400">{{ t('workspace.noLogs') }}</p>
      </div>
    </div>
    <!-- 底栏分页 -->
    <div class="h-9 px-4 flex items-center justify-between shrink-0 border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs">
      <span class="text-slate-400">{{ t('workspace.pageInfo', { page: currentPage, total: totalPages() }) }}</span>
      <div class="flex items-center gap-2">
        <button :disabled="currentPage <= 1" class="p-1 rounded text-slate-400 hover:text-slate-600 disabled:opacity-30 focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="prevPage"><ChevronLeft class="w-4 h-4" /></button>
        <span class="text-slate-500">{{ currentPage }} / {{ totalPages() }}</span>
        <button class="p-1 rounded text-slate-400 hover:text-slate-600 disabled:opacity-30 focus:ring-2 focus:ring-blue-400 focus:outline-none" @click="nextPage"><ChevronRight class="w-4 h-4" /></button>
      </div>
    </div>
  </div>
</template>
