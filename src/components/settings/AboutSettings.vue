<script setup lang="ts">
/** 关于设置——版本信息、安全提示、日志导出。 */
import { useI18n } from 'vue-i18n'

defineProps<{
  svnVersion: string
  repoUrl: string
}>()

const { t } = useI18n()
</script>

<template>
  <!-- 关于设置区块 -->
  <div class="max-w-xl">
    <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.about') }}</h3>
    <div class="space-y-2 text-xs text-slate-400 dark:text-slate-500">
      <p>Free-SVN 版本 0.0.1 · 基于 Tauri 2 · Apache 2.0 许可证</p>
      <!-- 有 SVN 版本时显示 -->
      <p v-if="svnVersion">{{ t('settings.svnVersion') }}: {{ svnVersion }}</p>
      <!-- 有仓库地址时显示 -->
      <p v-if="repoUrl">{{ t('settings.repositoryUrl') }}: <span class="font-mono">{{ repoUrl }}</span></p>

      <div class="mt-3 pt-3 border-t border-slate-200 dark:border-slate-700">
        <p class="text-xs font-medium text-slate-500 dark:text-slate-400 mb-2">{{ t('settings.securityTips') }}</p>
        <ul class="space-y-1 ml-4 list-disc">
          <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipPlaintext') }}</li>
          <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipClearAdvice') }}</li>
          <li class="text-amber-600 dark:text-amber-400">{{ t('settings.securityTipSessionOnly') }}</li>
        </ul>
      </div>

      <p class="text-amber-500 dark:text-amber-400 mt-2">{{ t('settings.securityNotice') }}</p>
    </div>
    <!-- 导出日志 -->
    <el-button size="small" class="mt-3 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none">{{ t('settings.exportLogs') }}</el-button>
  </div>
</template>
