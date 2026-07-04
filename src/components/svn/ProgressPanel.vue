<script setup lang="ts">
/**
 * 进度面板——非模态可拖拽进度弹窗。
 * 完全遵循设计文档 v4 4 布局，包含：可拖拽标题栏、<el-progress> 进度条、
 * 统计栏（四项横排——窄屏两行）、文件滚动区（<FileLineRow>）、底栏（自动关闭+取消）。
 */
import { ref, computed, watch, watchEffect, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProgressOverlay } from '@/composables/useProgressOverlay'
import { formatNumber } from '@/utils/format'
import FileLineRow from './FileLineRow.vue'

const { t } = useI18n()

// -- 从 composable 获取 UI 状态、进度数据和方法 --
const {
  isVisible, isCancelling, panelOffset,
  onDragStart, onDragMove, onDragEnd,
  cancelOperation, autoScrollToBottom, fileListRef,
  progress, fileLines,
} = useProgressOverlay()

// -- 标题栏格式化 --
const titleText = computed(() => {
  const op = progress.value?.operation || t('progress.operationInProgress')
  const pct = progress.value?.percent ?? 0
  return `${op}${t('progress.inProgress')} (${pct}%)`
})

// -- 待传输计数（预枚举文件总数 - 已完成数） --
const effectivePendingCount = computed(() => {
  const total = fileLines.value.length
  const completed = progress.value?.completedCount ?? 0
  return Math.max(0, total - completed)
})

// -- 本地 UI 状态 --
const overlayVisible = ref(false)       // 弹窗显示控制（含 500ms 延迟关闭）
let autoCloseTimer: ReturnType<typeof setTimeout> | null = null

// -- 弹窗显示逻辑（含 500ms 延迟关闭） --
watch(isVisible, (newVal) => {
  if (newVal) {
    overlayVisible.value = true
  } else {
    // 操作完成 -> 500ms 延迟关闭弹窗
    if (autoCloseTimer !== null) clearTimeout(autoCloseTimer)
    autoCloseTimer = setTimeout(() => {
      overlayVisible.value = false
    }, 500)
  }
})

// -- 取消操作 --
async function handleCancel() {
  await cancelOperation()
}

// -- 自动滚动 --
watchEffect(() => {
  if (fileLines.value.length > 0) {
    autoScrollToBottom()
  }
})

// -- 清理 --
onUnmounted(() => {
  if (autoCloseTimer !== null) clearTimeout(autoCloseTimer)
})
</script>

<template>
  <Teleport to="body">
    <!-- 非模态容器：pointer-events-none 点击穿透 -->
    <div
      v-if="overlayVisible"
      class="fixed inset-0 pointer-events-none z-50 flex items-start justify-center pt-[10vh]"
    >
      <!-- 可拖拽面板 -->
      <div
        class="pointer-events-auto w-[500px] max-w-[90vw] rounded-lg border border-slate-700 bg-slate-900 shadow-2xl select-none"
        :style="{
          transform: `translate(${panelOffset.x}px, ${panelOffset.y}px)`,
        }"
      >
        <!-- 标题栏（可拖拽） -->
        <div
          class="flex items-center justify-between px-4 py-3 bg-slate-800 rounded-t-lg cursor-move"
          @mousedown="onDragStart"
          @mousemove="onDragMove"
          @mouseup="onDragEnd"
        >
          <span class="text-sm font-semibold text-slate-100 truncate">
            {{ titleText }}
          </span>
        </div>

        <!-- 进度条 -->
        <div class="px-4 pt-4 pb-2">
          <el-progress
            :percentage="progress?.percent ?? 0"
            :stroke-width="12"
            striped
            striped-flow
            :duration="0.3"
          />
        </div>

        <!-- 统计栏（四项横排 -> 窄屏两行） -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-1 px-4 py-2 text-xs text-slate-400 font-mono">
          <!-- 传输速度 -->
          <div class="truncate">
            <span class="text-slate-500">{{ t('progress.downloadSpeed') }}:</span>
            {{ progress?.speed || '-' }}
          </div>
          <!-- 已完成文件数 -->
          <div class="truncate">
            <span class="text-slate-500">{{ t('progress.completedCount') }}:</span>
            {{ formatNumber(progress?.completedCount ?? 0) }}
          </div>
          <!-- 待传输估算 -->
          <div class="truncate">
            <span class="text-slate-500">{{ t('progress.pendingCount') }}:</span>
            {{ t('progress.pendingEstimate', { count: formatNumber(effectivePendingCount) }) }}
          </div>
          <!-- 已耗时间 -->
          <div class="truncate">
            <span class="text-slate-500">{{ t('progress.elapsed') }}:</span>
            {{ progress?.elapsed || '00:00' }}
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="mx-4 border-t border-slate-700"></div>

        <!-- 文件滚动区 -->
        <div
          ref="fileListRef"
          class="overflow-y-auto max-h-[280px] px-2 py-1"
        >
          <!-- 空状态：当前阶段文字 -->
          <div
            v-if="fileLines.length === 0"
            class="flex items-center justify-center h-20 text-sm text-slate-500"
          >
            {{ progress?.stage || t('progress.processing') }}
          </div>
          <!-- 文件行循环 -->
          <FileLineRow
            v-for="(line, index) in fileLines"
            :key="index"
            :filePath="line.filePath"
            :status="line.status"
          />
        </div>

        <!-- 分隔线 -->
        <div class="mx-4 border-t border-slate-700"></div>

        <!-- 底栏 -->
        <div class="flex items-center justify-between px-4 py-3">
          <span class="text-xs text-slate-500">{{ t('progress.autoClose') }}</span>
          <button
            class="px-4 py-1.5 rounded-md text-sm font-medium transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-slate-800"
            :class="isCancelling
              ? 'bg-slate-700 text-slate-400 cursor-not-allowed'
              : 'bg-red-800/40 text-red-400 hover:bg-red-800/60'"
            :disabled="isCancelling"
            @click="handleCancel"
          >
            {{ isCancelling ? t('progress.cancelling') : t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
