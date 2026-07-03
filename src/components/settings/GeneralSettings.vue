<script setup lang="ts">
/** 通用设置——开机自启动开关。 */
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const autoStart = defineModel<boolean>('autoStart', { required: true })
const emit = defineEmits<{ changed: [] }>()
</script>

<template>
  <div class="max-w-xl">
    <div class="flex items-center justify-between">
      <span class="text-sm text-slate-600 dark:text-slate-400">{{ t('settings.autoStart') }}</span>
      <el-switch v-model="autoStart" @change="emit('changed')" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" />
    </div>
  </div>
</template>
