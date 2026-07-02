<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { KeyRound, Eye, EyeOff, CheckCircle, XCircle, Loader2 } from 'lucide-vue-next'
import { testConnection, clearCredentials } from '@/services/svn'

const { t } = useI18n()

type AuthMode = 'retry' | 'manage'

interface Props {
  mode: AuthMode
  /** retry 模式下：失败操作的描述信息 */
  operationLabel?: string
  /** 仓库 URL */
  repoUrl?: string
  /** retry 模式下：要重新执行的函数（接收 credentials） */
  onRetry?: (username: string, password: string, saveToCache: boolean) => Promise<boolean>
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'retry',
  operationLabel: '',
  repoUrl: '',
})

const emit = defineEmits<{ close: []; success: [] }>()

// ── 表单状态 ──
const username = ref('')
const password = ref('')
const saveToCache = ref(false)
const showPassword = ref(false)
const isProcessing = ref(false)
const statusMessage = ref('')
const statusType = ref<'success' | 'error' | ''>('')

const canRetry = computed(() => username.value.trim().length > 0 && password.value.length > 0)
const dialogTitle = computed(() =>
  props.mode === 'retry' ? t('auth.authFailedTitle') : t('auth.credentialManagement')
)

// ── 重试（retry 模式） ──
async function handleRetry() {
  if (!canRetry.value || !props.onRetry) return
  isProcessing.value = true
  statusMessage.value = ''
  statusType.value = ''

  try {
    const ok = await props.onRetry(username.value, password.value, saveToCache.value)
    if (ok) {
      statusMessage.value = t('auth.operationSuccess')
      statusType.value = 'success'
      setTimeout(() => emit('success'), 800)
    } else {
      statusMessage.value = t('auth.authFailedMessage')
      statusType.value = 'error'
    }
  } catch (err: unknown) {
    statusMessage.value = err instanceof Error ? err.message : String(err)
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
  }
}

// ── 测试连接（manage 模式） ──
async function handleTestConnection() {
  if (!canRetry.value) return
  isProcessing.value = true
  statusMessage.value = ''
  statusType.value = ''

  try {
    const msg = await testConnection({ url: props.repoUrl || '', username: username.value, password: password.value })
    statusMessage.value = msg
    statusType.value = 'success'
  } catch (err: unknown) {
    statusMessage.value = err instanceof Error ? err.message : String(err)
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
  }
}

// ── 清除缓存（manage 模式） ──
async function handleClearCache() {
  isProcessing.value = true
  statusMessage.value = ''
  statusType.value = ''

  try {
    const msg = await clearCredentials(props.repoUrl || '')
    statusMessage.value = msg
    statusType.value = 'success'
  } catch (err: unknown) {
    statusMessage.value = err instanceof Error ? err.message : String(err)
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
  }
}
</script>

<template>
  <el-dialog
    :model-value="true"
    :title="dialogTitle"
    :width="mode === 'retry' ? '440px' : '480px'"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
    @close="emit('close')"
  >
    <div class="space-y-4">
      <div v-if="repoUrl" class="text-sm text-slate-600 dark:text-slate-400">
        <div class="font-medium text-slate-800 dark:text-slate-200">{{ t('auth.serverRequiresAuth') }}</div>
        <div class="mt-1 font-mono text-xs">{{ repoUrl }}</div>
        <div v-if="mode === 'retry' && operationLabel" class="text-xs text-slate-500 mt-0.5">
          {{ t('auth.operationPrefix') }}{{ operationLabel }}
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.username') }}</label>
        <el-input
          v-model="username"
          :placeholder="t('auth.usernamePlaceholder')"
          :disabled="isProcessing"
          size="default"
          @keyup.enter="mode === 'retry' ? handleRetry() : emit('close')"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.password') }}</label>
        <div class="relative">
          <el-input
            v-model="password"
            :type="showPassword ? 'text' : 'password'"
            :placeholder="t('auth.passwordPlaceholder')"
            :disabled="isProcessing"
            size="default"
            @keyup.enter="mode === 'retry' ? handleRetry() : emit('close')"
          />
          <button
            class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded"
            @click="showPassword = !showPassword"
            :aria-label="showPassword ? t('auth.hidePassword') : t('auth.showPassword')"
          >
            <EyeOff v-if="showPassword" class="w-4 h-4" />
            <Eye v-else class="w-4 h-4" />
          </button>
        </div>
      </div>

      <el-checkbox v-model="saveToCache" :disabled="isProcessing">
        <span class="text-sm">{{ t('auth.saveToCache') }}</span>
      </el-checkbox>

      <div
        v-if="statusMessage"
        class="flex items-center gap-2 text-sm px-3 py-2 rounded-md"
        :class="statusType === 'success'
          ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400'
          : 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400'"
      >
        <CheckCircle v-if="statusType === 'success'" class="w-4 h-4 shrink-0" />
        <XCircle v-else class="w-4 h-4 shrink-0" />
        <span>{{ statusMessage }}</span>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-2" :class="mode === 'manage' ? 'justify-between' : 'justify-end'">
        <div v-if="mode === 'manage'" class="flex items-center gap-2">
          <el-button size="small" :disabled="isProcessing || !canRetry" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleTestConnection">
            <Loader2 v-if="isProcessing" class="w-4 h-4 mr-1 animate-spin" />
            {{ t('auth.testConnection') }}
          </el-button>
          <el-button size="small" :disabled="isProcessing" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleClearCache">
            {{ t('auth.clearCache') }}
          </el-button>
        </div>

        <div class="flex items-center gap-2">
          <el-button class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')" :disabled="isProcessing">
            {{ mode === 'retry' ? t('auth.cancelOperation') : t('common.close') }}
          </el-button>
          <el-button
            v-if="mode === 'retry'"
            type="primary"
            :loading="isProcessing"
            :disabled="!canRetry"
            class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
            @click="handleRetry"
          >
            {{ isProcessing ? t('auth.retrying') : t('auth.retry') }}
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>
