<script setup lang="ts">
import { ref } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useFileSelection } from '@/composables/useFileSelection'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '@/stores/workspace'
import { open } from '@tauri-apps/plugin-shell'
import { join } from '@tauri-apps/api/path'
import { ElCheckbox, ElButton } from 'element-plus'
import FileStatusIcon from './FileStatusIcon.vue'
import FileActionButtons from './FileActionButtons.vue'
import type { FileItem } from '@/types/svn'

const { t } = useI18n()
const fileListStore = useFileListStore()
const workspaceStore = useWorkspaceStore()

// Shift+Click / Ctrl+Click 范围选择
const { handleClick: selectionHandleClick } = useFileSelection()

// 右键菜单状态
const ctxMenu = ref<{ show: boolean; x: number; y: number; file: FileItem | null }>({ show: false, x: 0, y: 0, file: null })

// 冲突行红色指示条
function rowClassName({ row }: { row: { status: string } }): string {
  return row.status === 'conflicted' ? 'border-l-[3px] border-red-500' : ''
}

function handleRowClick(row: FileItem, _column: unknown, event: MouseEvent) {
  const allFiles = fileListStore.filteredFiles
  const idx = allFiles.findIndex(f => f.path === row.path)
  if (idx >= 0) {
    selectionHandleClick(idx, event, allFiles, fileListStore.selectedPaths, (s) => { fileListStore.selectedPaths = s })
  }
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
function revertFile(path: string) { fileListStore.revertFile(path).catch(() => {}) }
function ignoreFile(path: string) { fileListStore.ignoreFile(path).catch(() => {}) }
function deleteFile(path: string) { fileListStore.deleteFile(path).catch(() => {}) }
function mergeFile(path: string) { emit('merge', path) }
function unlockFile(path: string) { fileListStore.unlockFile(path).catch(() => {}) }
</script>

<template>
  <div class="h-full flex flex-col">
    <el-table
      :data="fileListStore.filteredFiles"
      size="small" style="width:100%;height:100%" row-key="path"
      :row-class-name="rowClassName"
      @row-click="handleRowClick"
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
      </el-table-column>
      <el-table-column :label="t('file.status')" width="48" sortable="custom" prop="status">
        <template #default="{ row }"><FileStatusIcon :status="row.status" /></template>
      </el-table-column>
      <el-table-column :label="t('file.fileName')" min-width="200" sortable="custom" prop="path">
        <template #default="{ row }">
          <span class="truncate block text-sm font-mono text-slate-700 dark:text-slate-300" :title="row.path">{{ row.path }}</span>
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
      <ElCheckbox
        :model-value="fileListStore.selectedPaths.size === fileListStore.filteredFiles.length && fileListStore.filteredFiles.length > 0"
        :indeterminate="fileListStore.selectedPaths.size > 0 && fileListStore.selectedPaths.size < fileListStore.filteredFiles.length"
        @change="(v: boolean) => fileListStore.toggleSelectAll(v)"
      />
      <span class="text-slate-500 dark:text-slate-400">{{ t('workspace.selectedCount', { count: fileListStore.selectedPaths.size }) }}</span>
      <ElButton v-if="fileListStore.selectedPaths.size > 0" size="small" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none">
        {{ t('file.batchRevert') }}
      </ElButton>
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
