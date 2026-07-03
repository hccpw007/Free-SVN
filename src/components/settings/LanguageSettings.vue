<script setup lang="ts">
/** 语言设置——界面语言选择器。 */
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const language = defineModel<string>('language', { required: true })
const emit = defineEmits<{ changed: [] }>()
</script>

<template>
  <!-- 语言设置区块 -->
  <div class="max-w-xl">
    <div class="space-y-4">
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.language') }}</label>
        <!-- 界面语言选择器 -->
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
