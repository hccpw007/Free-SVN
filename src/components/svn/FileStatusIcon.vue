<script setup lang="ts">
defineProps<{ status: string }>()

const cfg: Record<string, { icon: string; cls: string }> = {
  modified:     { icon: 'M', cls: 'text-blue-500 dark:text-blue-400' },
  added:        { icon: '✚', cls: 'text-green-500 dark:text-green-400' },
  conflicted:   { icon: '!', cls: 'text-red-500 dark:text-red-400 font-bold border-l-[3px] border-red-500 pl-1' },
  deleted:      { icon: '🗑', cls: 'text-amber-500 dark:text-amber-400' },
  unversioned:  { icon: '?', cls: 'text-slate-400 dark:text-slate-500' },
  locked:       { icon: '-', cls: 'text-violet-500 dark:text-violet-400' },
}
</script>

<template>
  <span :class="cfg[status]?.cls" class="inline-flex items-center w-6 text-sm">{{ cfg[status]?.icon || '?' }}</span>
</template>
