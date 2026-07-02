<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'

const { t } = useI18n()

const defaultCheckoutDir = defineModel<string>('defaultCheckoutDir', { required: true })
const emit = defineEmits<{ changed: [] }>()

function markChanged() {
  emit('changed')
}

async function browseDir() {
  const selected = await open({ directory: true, title: t('dialog.selectTargetDir') })
  if (selected && typeof selected === 'string') {
    defaultCheckoutDir.value = selected
    markChanged()
  }
}
</script>

<template>
  <div class="max-w-xl">
    <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.svn') }}</h3>
    <div class="space-y-4">
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.defaultCheckoutDir') }}</label>
        <div class="flex gap-2 mt-1">
          <el-input v-model="defaultCheckoutDir" size="default" @input="markChanged" />
          <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="browseDir">{{ t('dialog.browse') }}</el-button>
        </div>
      </div>
    </div>
  </div>
</template>
