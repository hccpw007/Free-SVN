<script setup lang="ts">
/** 通用确认对话框——可配置标题/消息/文件列表/危险级别。 */
import { useI18n } from 'vue-i18n'

interface Props {
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  fileList?: string[]
  type?: 'danger' | 'warning' | 'info'
}
const props = withDefaults(defineProps<Props>(), {
  confirmText: '',
  cancelText: '',
  type: 'danger',
  fileList: () => [],
})
const emit = defineEmits<{ confirm: []; cancel: [] }>()
const { t } = useI18n()
</script>

<template>
  <!-- 确认对话框 -->
  <el-dialog :model-value="true" :title="title" width="420px" @close="emit('cancel')">
    <p class="text-sm text-slate-600 dark:text-slate-400">{{ message }}</p>
    <!-- 有文件时显示列表 -->
    <div v-if="fileList.length" class="mt-3 max-h-32 overflow-y-auto">
      <!-- 文件条目 -->
      <div v-for="f in fileList" :key="f" class="text-xs font-mono text-slate-600 dark:text-slate-400 py-0.5">{{ f }}</div>
    </div>
    <!-- 危险操作警告 -->
    <p v-if="type === 'danger'" class="text-xs text-red-500 mt-2">{{ t('file.irreversibleAction') }}</p>
    <!-- 取消/确认按钮组 -->
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('cancel')">{{ cancelText || t('common.cancel') }}</el-button>
      <el-button size="default" :type="type === 'danger' ? 'danger' : 'primary'" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('confirm')">{{ confirmText || t('common.confirm') }}</el-button>
    </template>
  </el-dialog>
</template>
