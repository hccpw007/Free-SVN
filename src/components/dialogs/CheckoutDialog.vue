<script setup lang="ts">
/** 检出对话框——SVN 仓库检出参数配置，含认证区域。 */
import { ref, reactive, computed, nextTick, onMounted } from 'vue'
import { useSvnStore } from '@/stores/svn'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { Eye, EyeOff } from 'lucide-vue-next'
import {ElMessage} from "element-plus";

const { t } = useI18n()
const svnStore = useSvnStore()
const workspaceStore = useWorkspaceStore()

const emit = defineEmits<{ close: [] }>()

const props = withDefaults(defineProps<{
  initialPath?: string
}>(), { initialPath: '' })

// URL 历史记录（localStorage 持久化）
const STORAGE_KEY_URLS = 'free-svn:recentUrls'
const MAX_RECENT_URLS = 10
const recentUrls = ref<string[]>([])

onMounted(() => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_URLS)
    if (stored) recentUrls.value = JSON.parse(stored)
  } catch { /* ignore */ }
})

function saveRecentUrl(url: string) {
  const idx = recentUrls.value.indexOf(url)
  if (idx >= 0) recentUrls.value.splice(idx, 1)
  recentUrls.value.unshift(url)
  if (recentUrls.value.length > MAX_RECENT_URLS) {
    recentUrls.value = recentUrls.value.slice(0, MAX_RECENT_URLS)
  }
  localStorage.setItem(STORAGE_KEY_URLS, JSON.stringify(recentUrls.value))
}

function querySearch(queryString: string, cb: (results: { value: string }[]) => void) {
  const results = queryString
    ? recentUrls.value.filter(url => url.toLowerCase().includes(queryString.toLowerCase()))
    : recentUrls.value
  cb(results.map(value => ({ value })))
}

// 表单状态
const repoUrl = ref('')
const targetPath = ref(props.initialPath)
const depth = ref('infinity')
const ignoreExternals = ref(false)
const emptyOnly = ref(false)
const isCheckingOut = ref(false)
// 认证失败后标记，用于展开/高亮认证区域
const authRetryVisible = ref(false)

// URL 校验
const urlValid = computed(() => /^svn:\/\/|^https:\/\/|^svn\+ssh:\/\//.test(repoUrl.value))
const urlError = computed(() => {
  if (!repoUrl.value) return ''
  if (!urlValid.value) return t('dialog.urlInvalid')
  return ''
})

// 路径浏览
async function browseTarget() {
  const selected = await open({ directory: true, title: t('dialog.selectTargetDir') })
  if (selected) targetPath.value = selected
}

// 认证区域
const authForm = reactive({
  username: '',
  password: '',
  saveToCache: false,
  showPassword: false,
})
const usernameError = ref('')
const passwordError = ref('')

function validateAuth(): boolean {
  let valid = true
  usernameError.value = ''
  passwordError.value = ''
  if (authForm.username && !authForm.username.trim()) {
    usernameError.value = t('error.usernameWhitespace')
    valid = false
  }
  if (authForm.username && !authForm.password) {
    passwordError.value = t('error.passwordEmpty')
    valid = false
  }
  return valid
}

/** 判断错误是否为认证相关（E170013 = 连接失败，非认证错误；E175013 = 无访问权限，需重新认证） */
function isAuthError(msg: string): boolean {
  return msg.includes('E215004') || msg.includes('E170001')
    || msg.includes('E175013')
    || msg.includes('Authentication failed') || msg.includes('认证失败')
    || msg.includes('No credentials') || msg.includes('authorization failed')
}

async function handleCheckout() {
  if (!urlValid.value || !validateAuth()) return
  isCheckingOut.value = true
  authRetryVisible.value = false
  try {
    const actualDepth = emptyOnly.value ? 'empty' : depth.value
    // 发起检出（不等待完成，弹窗立即关闭）
    svnStore.checkoutRepo({
      url: repoUrl.value,
      targetPath: targetPath.value,
      depth: actualDepth,
      ignoreExternals: ignoreExternals.value,
      credentials: authForm.username ? {
        username: authForm.username,
        password: authForm.password,
        saveToCache: authForm.saveToCache,
      } : undefined,
    }).then(result => {
      if (result) {
        workspaceStore.switchWorkspace(targetPath.value)
        saveRecentUrl(repoUrl.value)
      }
    }).catch((err: unknown) => {
      console.error('[checkout dialog] Error in checkout:', err)
      const msg = err instanceof Error ? err.message : String(err)
      ElMessage.error(msg)
    })
    // 弹窗立即关闭，进度由进度弹窗展示
    emit('close')
  } catch (err: unknown) {
    console.error('[checkout dialog] Error in checkout:', err)
    const msg = err instanceof Error ? err.message : String(err)
    // 如果是认证错误，展开认证区域引导用户输入凭据
    if (isAuthError(msg)) {
      authRetryVisible.value = true
      ElMessage.warning(t('error.SVN_AUTH_FAILED'))
      nextTick(() => {
        document.querySelector<HTMLInputElement>('#checkout-username')?.focus()
      })
    } else {
      ElMessage.error(msg)
    }
  } finally {
    isCheckingOut.value = false
  }
}
</script>

<template>
  <!-- 检出对话框 -->
  <el-dialog :model-value="true" :title="t('dialog.checkout')" width="480px"
    :close-on-click-modal="false" :close-on-press-escape="!isCheckingOut"
    @close="emit('close')">
    <!-- 表单内容区 -->
    <div class="flex flex-col gap-3">
      <!-- 仓库 URL -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.repoUrl') }}</label>
        <el-autocomplete
          v-model="repoUrl"
          :fetch-suggestions="querySearch"
          :trigger-on-focus="true"
          size="small"
          clearable
          class="!w-full"
          :placeholder="t('dialog.repoUrlPlaceholder')"
        />
        <!-- URL 格式错误提示 -->
        <p v-if="urlError" class="text-xs text-red-500 mt-1">{{ urlError }}</p>
      </div>

      <!-- 目标路径 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.targetPath') }}</label>
        <div class="flex gap-2">
          <el-input v-model="targetPath" size="small" :placeholder="t('dialog.selectTargetDir')" />
          <el-button size="small" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="browseTarget">{{ t('dialog.browse') }}</el-button>
        </div>
      </div>

      <!-- 深度选项 -->
      <div class="flex gap-3 items-center">
        <div class="flex-1">
          <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.depth') }}</label>
          <el-select v-model="depth" size="small" :disabled="emptyOnly">
            <el-option value="infinity" :label="t('dialog.depthInfinity')" />
            <el-option value="immediates" :label="t('dialog.depthImmediates')" />
            <el-option value="children" :label="t('dialog.depthChildren')" />
            <el-option value="empty" :label="t('dialog.depthEmpty')" />
          </el-select>
        </div>
      </div>

      <!-- 忽略外部引用 -->
      <el-checkbox v-model="ignoreExternals" size="small">{{ t('dialog.ignoreExternals') }}</el-checkbox>
      <!-- 仅检出当前目录 -->
      <el-checkbox v-model="emptyOnly" size="small">{{ t('dialog.currentDirOnly') }}</el-checkbox>

      <!-- 认证区域 -->
      <div class="border-t pt-3" :class="authRetryVisible ? 'border-red-400 dark:border-red-500' : 'border-slate-200 dark:border-slate-700'">
        <label class="text-xs font-medium mb-2 block" :class="authRetryVisible ? 'text-red-600 dark:text-red-400' : 'text-slate-700 dark:text-slate-300'">
          {{ authRetryVisible ? t('error.SVN_AUTH_FAILED') : t('dialog.authOptional') }}
        </label>
        <!-- 认证失败提示 -->
        <div v-if="authRetryVisible" class="text-xs text-red-500 dark:text-red-400 mb-2">
          {{ t('auth.authFailedTitle') }}
        </div>
        <div class="flex flex-col gap-2">
          <div>
            <el-input id="checkout-username" v-model="authForm.username" size="small" :placeholder="t('auth.username')" />
            <!-- 用户名错误提示 -->
            <p v-if="usernameError" class="text-xs text-red-500 mt-1">{{ usernameError }}</p>
          </div>
          <div>
            <div class="relative">
              <el-input v-model="authForm.password" size="small"
                :type="authForm.showPassword ? 'text' : 'password'"
                :placeholder="t('auth.password')" />
              <!-- 切换密码可见性 -->
              <button
                @click="authForm.showPassword = !authForm.showPassword"
                class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500 hover:text-slate-600 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded"
                :aria-label="authForm.showPassword ? t('auth.hidePassword') : t('auth.showPassword')"
              >
                <EyeOff v-if="authForm.showPassword" class="w-4 h-4" />
                <Eye v-else class="w-4 h-4" />
              </button>
            </div>
            <!-- 密码错误提示 -->
            <p v-if="passwordError" class="text-xs text-red-500 mt-1">{{ passwordError }}</p>
          </div>
          <!-- 缓存凭证 -->
          <el-checkbox v-model="authForm.saveToCache" size="small">{{ t('auth.saveToCache') }}</el-checkbox>
        </div>
      </div>
    </div>

    <!-- 底部按钮区 -->
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')" :disabled="isCheckingOut">
        {{ t('dialog.cancel') }}
      </el-button>
      <el-button size="default" type="primary" :loading="isCheckingOut"
        :disabled="!urlValid || !targetPath" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleCheckout">
        {{ t('dialog.checkout') }}
      </el-button>
    </template>
  </el-dialog>
</template>
