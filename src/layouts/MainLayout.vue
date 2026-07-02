<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useAutoRefresh } from '@/composables/useAutoRefresh'
import TopBar from '@/components/common/TopBar.vue'
import ToolBar from '@/components/common/ToolBar.vue'
import StatusBar from '@/components/common/StatusBar.vue'
import ProgressOverlay from '@/components/svn/ProgressOverlay.vue'

const route = useRoute()

// 窗口焦点变化时自动刷新文件列表（300ms 防抖）
useAutoRefresh()

// 欢迎页模式：route.name === 'WelcomePage' 时隐藏 ToolBar 和 StatusBar
const isWelcomePage = computed(() => {
  return route.name === 'WelcomePage'
})
</script>

<template>
  <div class="h-screen flex flex-col bg-slate-50 dark:bg-slate-900">
    <!-- TopBar: 48px 固定高度 -->
    <TopBar class="shrink-0" />

    <!-- ToolBar: 40px 固定高度，欢迎页模式隐藏 -->
    <ToolBar v-if="!isWelcomePage" class="shrink-0" />

    <!-- RouterView: 弹性填充，溢出滚动 -->
    <main class="flex-1 min-h-0 overflow-auto text-slate-900 dark:text-slate-100 p-4">
      <RouterView />
    </main>

    <!-- StatusBar: 28px 固定高度，欢迎页模式隐藏 -->
    <StatusBar v-if="!isWelcomePage" class="shrink-0" />

    <!-- ProgressOverlay: 全屏遮罩，通过 v-if 控制显示 -->
    <ProgressOverlay />
  </div>
</template>
