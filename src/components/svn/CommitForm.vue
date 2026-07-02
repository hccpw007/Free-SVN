<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Store } from '@tauri-apps/plugin-store'
import { useI18n } from 'vue-i18n'

const model = defineModel<string>({ required: true })
const { t } = useI18n()
const charCount = ref(0)
const recentMessages = ref<string[]>([])
const selectedRecentMsg = ref('')

watch(model, (v) => { charCount.value = v.length })

// 职责边界（与 CommitPage 严格分离）：
// - CommitPage（父组件）：持久化写入 → saveRecentMessage() 调用 tauri-plugin-store.set
// - CommitForm（此组件）：仅读取和展示 → onMounted 调用 tauri-plugin-store.get
// CommitForm 不执行任何持久化写入操作，写入全部委托给父组件 CommitPage
const RECENT_MSG_KEY = 'recentCommitMessages'
onMounted(async () => {
  try {
    const store = await Store.load('settings.json')
    const saved = await store.get<string[]>(RECENT_MSG_KEY)
    recentMessages.value = saved || []
  } catch { /* 读取失败不影响提交流程 */ }
})

function selectRecent(msg: string) { model.value = msg }
</script>

<template>
  <div class="shrink-0 px-4 py-3 border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800">
    <!-- 最近使用日志下拉 -->
    <div v-if="recentMessages.length > 0" class="mb-2">
      <el-select v-model="selectedRecentMsg" :placeholder="t('file.recentMessages')" size="small" class="!w-full" @change="selectRecent">
        <el-option v-for="msg in recentMessages" :key="msg" :label="msg.slice(0, 60) + (msg.length > 60 ? '...' : '')" :value="msg" />
      </el-select>
    </div>
    <textarea
      :value="model"
      @input="model = ($event.target as HTMLTextAreaElement).value"
      :placeholder="t('file.commitPlaceholder')"
      class="w-full min-h-[80px] max-h-[200px] p-3 text-xs font-mono rounded-md border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200 resize-y focus:outline-none focus:ring-2 focus:ring-green-500/30 focus:border-green-500 transition-colors"
      :class="model.split('\n').some(l => l.length > 72) ? '!border-amber-400' : ''"
    />
    <div class="mt-1 flex items-center justify-between text-xs">
      <span class="text-slate-400">{{ t('file.charCount', { count: charCount }) }}</span>
    </div>
  </div>
</template>
