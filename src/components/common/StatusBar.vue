<script setup lang="ts">
import { computed } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const ws = useWorkspaceStore()

const revision = computed(() => ws.currentRevision ? t('statusBar.revision', { rev: ws.currentRevision }) : t('statusBar.revisionNone'))
const commitTime = computed(() => ws.lastCommitTime ? t('statusBar.lastCommit', { time: ws.lastCommitTime }) : '')
const svnVer = computed(() => ws.svnVersion ? t('statusBar.svnVersion', { ver: ws.svnVersion }) : '')

async function copyRev() {
  if (ws.currentRevision) {
    await navigator.clipboard.writeText(String(ws.currentRevision))
  }
}
</script>

<template>
  <div class="h-7 px-4 flex items-center justify-between bg-slate-100 dark:bg-slate-800/50 border-t border-slate-200 dark:border-slate-700 text-xs">
    <span
      class="text-slate-500 dark:text-slate-400 font-mono cursor-pointer hover:text-green-600 transition-colors duration-150"
      :title="t('statusBar.copyRevision')"
      @click="copyRev"
    >{{ revision }}</span>
    <span v-if="commitTime" class="text-slate-500 dark:text-slate-400 font-mono" :title="commitTime">{{ commitTime }}</span>
    <span v-if="svnVer" class="text-slate-400 dark:text-slate-500 font-mono" :title="svnVer">{{ svnVer }}</span>
  </div>
</template>
