<script setup lang="ts">
// AuthDialog 占位组件 — TopBar 依赖
// 完整实现在后续阶段（6D 对话框阶段）中完成
interface Props {
  mode: 'manage' | 'auth'
  repoUrl?: string
}
defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <!-- 占位组件：由 TopBar v-if 控制显示，后续阶段实现完整功能 -->
</template>
