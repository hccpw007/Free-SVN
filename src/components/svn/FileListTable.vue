<script setup lang="ts">
/** 文件列表表格——变更文件列表、右键菜单、状态标签、排序/筛选。 */
import { ref } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '@/stores/workspace'
import { open } from '@tauri-apps/plugin-shell'
import { join } from '@tauri-apps/api/path'
import { ElCheckbox } from 'element-plus'
import FileActionButtons from './FileActionButtons.vue'
import type { FileItem } from '@/types/svn'

const { t } = useI18n()
const fileListStore = useFileListStore()
const workspaceStore = useWorkspaceStore()

// 右键菜单状态
const ctxMenu = ref<{ show: boolean; x: number; y: number; file: FileItem | null }>({ show: false, x: 0, y: 0, file: null })

// 冲突行红色指示条
function rowClassName({ row }: { row: { status: string } }): string {
  return row.status === 'conflicted' ? 'border-l-[3px] border-red-500' : ''
}

// 状态 tag 配置：文字、标签颜色、悬停说明 i18n key
const statusCfg: Record<string, { labelKey: string; tagCls: string; descKey: string }> = {
  modified:     { labelKey: 'file.statusModified',     tagCls: 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300',     descKey: 'file.descModified' },
  added:        { labelKey: 'file.statusPendingAdd',    tagCls: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300',  descKey: 'file.descPendingAdd' },
  conflicted:   { labelKey: 'file.statusConflicted',   tagCls: 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300',        descKey: 'file.descConflicted' },
  deleted:      { labelKey: 'file.statusPendingDelete', tagCls: 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300', descKey: 'file.descPendingDelete' },
  unversioned:  { labelKey: 'file.statusPendingAdd',    tagCls: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300',  descKey: 'file.descPendingAdd' },
  ignored:      { labelKey: 'file.statusIgnored',      tagCls: 'bg-slate-100 text-slate-400 dark:bg-slate-800 dark:text-slate-500 italic', descKey: 'file.descIgnored' },
  missing:      { labelKey: 'file.statusPendingDelete', tagCls: 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300', descKey: 'file.descPendingDelete' },
  replaced:     { labelKey: 'file.statusPendingAdd',    tagCls: 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300',  descKey: 'file.descPendingAdd' },
  obstructed:   { labelKey: 'file.statusAbnormal',      tagCls: 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-300',        descKey: 'file.descAbnormal' },
  external:     { labelKey: 'file.statusIgnored',       tagCls: 'bg-slate-100 text-slate-400 dark:bg-slate-800 dark:text-slate-500 italic', descKey: 'file.descIgnored' },
  incomplete:   { labelKey: 'file.statusAbnormal',      tagCls: 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-300',        descKey: 'file.descAbnormal' },
}

function statusLabel(status: string): string {
  return t(statusCfg[status]?.labelKey || status)
}

function statusDesc(status: string): string {
  return t(statusCfg[status]?.descKey || '')
}

/** 格式化文件大小（保留一位小数） */
function formatSize(bytes?: number): string {
  if (bytes === undefined || bytes === null) return '-'
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const val = bytes / Math.pow(1024, i)
  return val.toFixed(1) + ' ' + units[i]
}



function handleContextMenu(e: MouseEvent, row: FileItem) {
  e.preventDefault()
  ctxMenu.value = { show: true, x: e.clientX, y: e.clientY, file: row }
}

async function copyFullPath(p: string) {
  const absolutePath = await join(workspaceStore.currentPath, p)
  try { await navigator.clipboard.writeText(absolutePath) } catch { /* fallback */ }
  ctxMenu.value.show = false
}

async function copyRelativePath(p: string) {
  try { await navigator.clipboard.writeText(p) } catch { /* fallback */ }
  ctxMenu.value.show = false
}

async function showInExplorer(p: string) {
  const fullPath = await join(workspaceStore.currentPath, p)
  try { await open(fullPath) } catch { /* fallback */ }
  ctxMenu.value.show = false
}

async function openWithEditor(p: string) {
  const fullPath = await join(workspaceStore.currentPath, p)
  try { await open(fullPath) } catch { /* fallback */ }
  ctxMenu.value.show = false
}

// 事件冒泡架构: FileActionButtons → FileListTable(转发) → HomePage
const emit = defineEmits<{ diff: [path: string]; merge: [path: string] }>()
function diffFile(path: string) { emit('diff', path) }
function revertFile(path: string) { fileListStore.revertFile(path).catch(e => console.error('[FileListTable] revertFile 失败:', e)) }
function ignoreFile(path: string) { fileListStore.ignoreFile(path).catch(e => console.error('[FileListTable] ignoreFile 失败:', e)) }
function deleteFile(path: string) { fileListStore.deleteFile(path).catch(e => console.error('[FileListTable] deleteFile 失败:', e)) }
function mergeFile(path: string) { emit('merge', path) }
function unlockFile(path: string) { fileListStore.unlockFile(path).catch(e => console.error('[FileListTable] unlockFile 失败:', e)) }
</script>

<template>
  <div class="h-full flex flex-col">
    <el-table
      :data="fileListStore.filteredFiles"
      size="small" style="width:100%;height:100%" row-key="path"
      :row-class-name="rowClassName"
      @cell-contextmenu="(r: FileItem) => handleContextMenu($event, r)"
      @sort-change="({ prop, order }: { prop: string | null; order: string | null }) => {
        fileListStore.sortField = prop || 'status'
        fileListStore.sortOrder = order === 'ascending' ? 'asc' : 'desc'
      }"
      v-loading="fileListStore.isLoading"
    >
      <el-table-column width="40">
        <template #default="{ row }">
          <ElCheckbox :model-value="fileListStore.selectedPaths.has(row.path)" @change="() => fileListStore.toggleSelect(row.path)" />
        </template>
        <template #header>
          <ElCheckbox
            :model-value="fileListStore.selectedPaths.size === fileListStore.filteredFiles.length && fileListStore.filteredFiles.length > 0"
            :indeterminate="fileListStore.selectedPaths.size > 0 && fileListStore.selectedPaths.size < fileListStore.filteredFiles.length"
            @change="(v: boolean) => fileListStore.toggleSelectAll(v)"
          />
        </template>
      </el-table-column>
      <el-table-column :label="t('file.status')" width="80" sortable="custom" prop="status">
        <template #default="{ row }">
          <el-tooltip :content="statusDesc(row.status)" placement="top" :show-after="300" effect="dark">
            <span class="inline-block px-2 py-0.5 rounded text-xs font-medium cursor-default leading-5" :class="statusCfg[row.status]?.tagCls || 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'">
              {{ statusLabel(row.status) }}
            </span>
          </el-tooltip>
        </template>
      </el-table-column>
      <el-table-column :label="t('file.fileName')" min-width="200" sortable="custom" prop="path">
        <template #default="{ row }">
          <span class="truncate block text-sm font-mono text-slate-700 dark:text-slate-300" :title="row.path">{{ row.path }}</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('file.fileSize')" width="100" sortable="custom" prop="size" align="right">
        <template #default="{ row }">
          <span class="text-sm text-slate-600 dark:text-slate-400 tabular-nums">{{ formatSize(row.size) }}</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('file.actions')" width="180" fixed="right">
        <template #default="{ row }">
          <FileActionButtons :file="row" @diff="diffFile" @revert="revertFile" @ignore="ignoreFile" @delete="deleteFile" @merge="mergeFile" @unlock="unlockFile" />
        </template>
      </el-table-column>
    </el-table>

    <!-- 固定底栏 -->
    <div class="shrink-0 h-9 px-4 flex items-center gap-3 text-xs border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800">
      <span class="text-slate-500 dark:text-slate-400">{{ t('workspace.selectedCount', { count: fileListStore.selectedPaths.size }) }}</span>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="ctxMenu.show"
        class="fixed z-50 w-48 py-1 rounded-md bg-white dark:bg-slate-800 shadow-lg border border-slate-200 dark:border-slate-700 text-xs transition-all duration-200 ease-out"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" @click.stop
      >
        <button class="w-full px-3 py-1.5 text-left text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="copyFullPath(ctxMenu.file.path)">{{ t('file.copyFullPath') }}</button>
        <button class="w-full px-3 py-1.5 text-left text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="copyRelativePath(ctxMenu.file.path)">{{ t('file.copyRelativePath') }}</button>
        <div class="border-t border-slate-200 dark:border-slate-700 my-1" />
        <button class="w-full px-3 py-1.5 text-left text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="showInExplorer(ctxMenu.file.path)">{{ t('file.showInExplorer') }}</button>
        <button class="w-full px-3 py-1.5 text-left text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-700 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="openWithEditor(ctxMenu.file.path)">{{ t('file.openWithEditor') }}</button>
      </div>
      <!-- 关闭层 -->
      <div v-if="ctxMenu.show" class="fixed inset-0 z-40" @click="ctxMenu.show = false" />
    </Teleport>
  </div>
</template>
