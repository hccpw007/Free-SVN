<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const ignorePattern = defineModel<string>('ignorePattern', { required: true })
const emit = defineEmits<{ changed: [] }>()

function markChanged() {
  emit('changed')
}
</script>

<template>
  <div class="max-w-2xl">
    <div class="space-y-4">
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.ignorePatternDescription') }}</label>
        <el-input
          v-model="ignorePattern"
          type="textarea"
          :rows="20"
          class="mt-2"
          @input="markChanged"
        />
      </div>
    </div>
  </div>
</template>
