<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const darkMode = defineModel<boolean>('darkMode', { required: true })
const showUnversioned = defineModel<boolean>('showUnversioned', { required: true })
const language = defineModel<string>('language', { required: true })
const emit = defineEmits<{ changed: [] }>()
</script>

<template>
  <div class="max-w-xl">
    <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.interface') }}</h3>
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.darkMode') }}</span>
        <el-switch v-model="darkMode" @change="emit('changed')" />
      </div>
      <div class="flex items-center justify-between">
        <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.showUnversioned') }}</span>
        <el-switch v-model="showUnversioned" @change="emit('changed')" />
      </div>
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.language') }}</label>
        <el-select v-model="language" size="default" class="!w-full mt-1" @change="emit('changed')">
          <el-option :label="t('settings.langSystem')" value="system" />
          <el-option :label="t('settings.langZhCN')" value="zh-CN" />
          <el-option :label="t('settings.langEn')" value="en" />
          <el-option :label="t('settings.langJa') + ' (Coming Soon)'" value="ja" />
          <el-option :label="t('settings.langKo') + ' (Coming Soon)'" value="ko" />
        </el-select>
      </div>
    </div>
  </div>
</template>
