<script setup lang="ts">
/** 差异对比页面——包装 DiffViewer 组件作为独立路由。 */
import DiffViewer from '@/components/svn/DiffViewer.vue'
</script>

<!-- diff 页面包装层 -->
<template>
  <!-- 差异对比渲染容器 -->
  <div class="h-full w-full">
    <DiffViewer />
  </div>
</template>
