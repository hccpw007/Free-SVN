<script setup lang="ts">
defineProps<{ status: string }>()

const cfg: Record<string, { icon: string; cls: string }> = {
  modified:     { icon: 'M', cls: 'text-blue-500 dark:text-blue-400' },
  added:        { icon: '✚', cls: 'text-green-500 dark:text-green-400' },
  conflicted:   { icon: '!', cls: 'text-red-500 dark:text-red-400 font-bold' },
  deleted:      { icon: '🗑', cls: 'text-amber-500 dark:text-amber-400' },
  unversioned:  { icon: '?', cls: 'text-slate-400 dark:text-slate-500' },
  ignored:      { icon: 'I', cls: 'text-slate-300 dark:text-slate-600 italic' },
  missing:      { icon: '⚠', cls: 'text-orange-500 dark:text-orange-400' },
  replaced:     { icon: 'R', cls: 'text-cyan-500 dark:text-cyan-400' },
  obstructed:   { icon: '~', cls: 'text-orange-400 dark:text-orange-300' },
  external:     { icon: 'X', cls: 'text-violet-400 dark:text-violet-300' },
  incomplete:   { icon: '⚠', cls: 'text-red-400 dark:text-red-300' },
  normal:       { icon: '✓', cls: 'text-green-400 dark:text-green-500' },
}
</script>

<template>
  <span :class="cfg[status]?.cls" class="inline-flex items-center w-6 text-sm">{{ cfg[status]?.icon || '?' }}</span>
</template>
