<script setup lang="ts">
/** SVN 设置——默认检出目录、差异/合并工具选择。 */
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { computed } from 'vue'

const { t } = useI18n()

const defaultCheckoutDir = defineModel<string>('defaultCheckoutDir', { required: true })
const diffTool = defineModel<string>('diffTool', { required: true })
const diffCommandTemplate = defineModel<string>('diffCommandTemplate', { required: true })
const mergeTool = defineModel<string>('mergeTool', { required: true })
const mergeCommandTemplate = defineModel<string>('mergeCommandTemplate', { required: true })
const fallbackToBuiltin = defineModel<boolean>('fallbackToBuiltin', { required: true })
const emit = defineEmits<{ changed: [] }>()

const showDiffCustom = computed(() => diffTool.value === 'custom')
const showMergeCustom = computed(() => mergeTool.value === 'custom')

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
  <!-- SVN 设置 -->
  <div class="max-w-xl">
    <div class="space-y-6">
      <!-- 默认检出目录 -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.defaultCheckoutDir') }}</label>
        <div class="flex gap-2 mt-1">
          <el-input v-model="defaultCheckoutDir" size="default" @input="markChanged" />
          <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="browseDir">{{ t('dialog.browse') }}</el-button>
        </div>
      </div>

      <!-- 分区线 -->
      <el-divider />

      <!-- 差异/合并工具 -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffTool') }}</label>
        <el-select v-model="diffTool" size="default" class="!w-full mt-1" @change="markChanged">
          <el-option label="内置 (diff2html)" value="builtin" />
          <el-option label="VS Code" value="vscode" />
          <el-option label="Beyond Compare" value="beyond_compare" />
          <el-option label="Kaleidoscope" value="kaleidoscope" />
          <el-option label="自定义" value="custom" />
        </el-select>
      </div>
      <!-- 自定义差异命令 -->
      <div v-if="showDiffCustom">
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.diffCommandTemplate') }}</label>
        <el-input v-model="diffCommandTemplate" size="default" placeholder="code --diff <file1> <file2>" class="mt-1" @input="markChanged" />
      </div>
      <!-- 合并工具选择 -->
      <div>
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeTool') }}</label>
        <el-select v-model="mergeTool" size="default" class="!w-full mt-1" @change="markChanged">
          <el-option label="内置 (差异对比+标记)" value="builtin" />
          <el-option label="VS Code" value="vscode" />
          <el-option label="Beyond Compare" value="beyond_compare" />
          <el-option label="Kaleidoscope" value="kaleidoscope" />
          <el-option label="自定义" value="custom" />
        </el-select>
      </div>
      <!-- 自定义合并命令 -->
      <div v-if="showMergeCustom">
        <label class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.mergeCommandTemplate') }}</label>
        <el-input v-model="mergeCommandTemplate" size="default" placeholder="bcomp <mine> <base> <theirs> <output>" class="mt-1" @input="markChanged" />
      </div>
      <!-- 回退到内置工具 -->
      <el-checkbox v-model="fallbackToBuiltin" @change="markChanged">
        <span class="text-xs">{{ t('settings.fallbackToBuiltin') }}</span>
      </el-checkbox>
    </div>
  </div>
</template>
