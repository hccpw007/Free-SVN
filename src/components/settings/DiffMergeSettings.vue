<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'

const { t } = useI18n()

const diffTool = defineModel<string>('diffTool', { required: true })
const diffCommandTemplate = defineModel<string>('diffCommandTemplate', { required: true })
const mergeTool = defineModel<string>('mergeTool', { required: true })
const mergeCommandTemplate = defineModel<string>('mergeCommandTemplate', { required: true })
const fallbackToBuiltin = defineModel<boolean>('fallbackToBuiltin', { required: true })
const emit = defineEmits<{ changed: [] }>()

const showDiffCustom = computed(() => diffTool.value === 'custom')
const showMergeCustom = computed(() => mergeTool.value === 'custom')
</script>

<template>
  <div class="max-w-xl">
    <h3 class="text-base font-medium text-slate-800 dark:text-slate-200 mb-4">{{ t('settings.diffMerge') }}</h3>
    <div class="space-y-4">
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffTool') }}</label>
        <el-select v-model="diffTool" size="default" class="!w-full mt-1" @change="emit('changed')">
          <el-option label="内置 (diff2html)" value="builtin" />
          <el-option label="VS Code" value="vscode" />
          <el-option label="Beyond Compare" value="beyond_compare" />
          <el-option label="Kaleidoscope" value="kaleidoscope" />
          <el-option label="自定义" value="custom" />
        </el-select>
      </div>
      <div v-if="showDiffCustom">
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffCommandTemplate') }}</label>
        <el-input v-model="diffCommandTemplate" size="default" placeholder="code --diff <file1> <file2>" class="mt-1" @input="emit('changed')" />
      </div>
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeTool') }}</label>
        <el-select v-model="mergeTool" size="default" class="!w-full mt-1" @change="emit('changed')">
          <el-option label="内置 (差异对比+标记)" value="builtin" />
          <el-option label="VS Code" value="vscode" />
          <el-option label="Beyond Compare" value="beyond_compare" />
          <el-option label="Kaleidoscope" value="kaleidoscope" />
          <el-option label="自定义" value="custom" />
        </el-select>
      </div>
      <div v-if="showMergeCustom">
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeCommandTemplate') }}</label>
        <el-input v-model="mergeCommandTemplate" size="default" placeholder="bcomp <mine> <base> <theirs> <output>" class="mt-1" @input="emit('changed')" />
      </div>
      <el-checkbox v-model="fallbackToBuiltin" @change="emit('changed')">
        <span class="text-xs">{{ t('settings.fallbackToBuiltin') }}</span>
      </el-checkbox>
    </div>
  </div>
</template>
