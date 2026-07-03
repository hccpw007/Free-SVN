<script setup lang="ts">
/** 账号管理设置——缓存的 SVN 凭据列表管理与增删改。 */
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Pencil, Trash2 } from 'lucide-vue-next'
import { listCachedCredentials, saveCredentials, deleteCachedCredential, updateCachedCredentialPassword } from '@/services/svn'
import type { CachedCredential } from '@/types/svn'

const { t } = useI18n()

const cachedCredentials = ref<CachedCredential[]>([])
const isLoadingAccounts = ref(false)
const showAddDialog = ref(false)
const showEditDialog = ref(false)
const editingCredential = ref<CachedCredential | null>(null)
const addForm = reactive({ url: '', username: '', password: '' })
const isAdding = ref(false)
const editForm = reactive({ username: '', newPassword: '' })
const isEditing = ref(false)

async function loadCachedCredentials() {
  isLoadingAccounts.value = true
  try {
    cachedCredentials.value = await listCachedCredentials()
  } catch (e: unknown) {
    console.warn('[AccountManagementSettings] 加载缓存凭据失败:', e)
  } finally {
    isLoadingAccounts.value = false
  }
}

async function handleAddAccount() {
  if (!addForm.url || !addForm.username || !addForm.password) return
  isAdding.value = true
  try {
    await saveCredentials({ url: addForm.url, username: addForm.username, password: addForm.password })
    ElMessage.success(t('settings.accountSaved'))
    showAddDialog.value = false
    addForm.url = ''
    addForm.username = ''
    addForm.password = ''
    await loadCachedCredentials()
  } catch (e: unknown) {
    ElMessage.error(typeof e === 'string' ? e : e instanceof Error ? e.message : String(e))
  } finally {
    isAdding.value = false
  }
}

async function handleDeleteCredential(cred: CachedCredential) {
  try {
    await ElMessageBox.confirm(
      t('settings.confirmDeleteAccount', { username: cred.username }),
      t('common.confirm'),
      { confirmButtonText: t('common.delete'), cancelButtonText: t('common.cancel'), type: 'warning' },
    )
    await deleteCachedCredential(cred.id)
    ElMessage.success(t('settings.accountDeleted'))
    await loadCachedCredentials()
  } catch { /* cancel */ }
}

function openEditDialog(cred: CachedCredential) {
  editingCredential.value = cred
  editForm.username = cred.username
  editForm.newPassword = ''
  showEditDialog.value = true
}

async function handleEditPassword() {
  if (!editingCredential.value || !editForm.newPassword) return
  isEditing.value = true
  try {
    await updateCachedCredentialPassword(editingCredential.value.id, editForm.newPassword)
    ElMessage.success(t('settings.passwordUpdated'))
    showEditDialog.value = false
    editingCredential.value = null
    await loadCachedCredentials()
  } catch (e: unknown) {
    ElMessage.error(typeof e === 'string' ? e : e instanceof Error ? e.message : String(e))
  } finally {
    isEditing.value = false
  }
}

function openAddDialog() {
  addForm.url = ''
  addForm.username = ''
  addForm.password = ''
  showAddDialog.value = true
}

onMounted(loadCachedCredentials)
</script>

<template>
  <!-- 账号管理设置 -->
  <div class="max-w-2xl">
    <div>
      <!-- 标题栏与添加按钮 -->
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-base font-medium text-slate-800 dark:text-slate-200">{{ t('settings.account') }}</h3>
        <el-button size="small" type="primary" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="openAddDialog">
          <Plus class="w-4 h-4 mr-1" />{{ t('settings.addAccount') }}
        </el-button>
      </div>

      <!-- 加载中 -->
      <div v-if="isLoadingAccounts" class="py-12 text-center text-sm text-slate-400 dark:text-slate-500">
        {{ t('common.loading') }}
      </div>

      <!-- 无凭据空状态 -->
      <div v-else-if="cachedCredentials.length === 0" class="py-12 text-center text-sm text-slate-400 dark:text-slate-500">
        {{ t('settings.noSavedAccounts') }}
      </div>

      <!-- 凭据表格 -->
      <div v-else class="border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-slate-50 dark:bg-slate-800/50 text-slate-500 dark:text-slate-400">
              <th class="text-left px-4 py-2.5 font-medium">{{ t('auth.username') }}</th>
              <th class="text-left px-4 py-2.5 font-medium">{{ t('settings.accountRealm') }}</th>
              <th class="text-right px-4 py-2.5 font-medium">{{ t('settings.accountActions') }}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200 dark:divide-slate-700">
            <tr v-for="cred in cachedCredentials" :key="cred.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/30">
              <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{{ cred.username }}</td>
              <td class="px-4 py-3 text-slate-500 dark:text-slate-400 font-mono text-xs">{{ cred.realm }}</td>
              <td class="px-4 py-3 text-right">
                <div class="flex items-center justify-end gap-1">
                  <el-button size="small" text class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="openEditDialog(cred)">
                    <Pencil class="w-4 h-4" />
                  </el-button>
                  <el-button size="small" text type="danger" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleDeleteCredential(cred)">
                    <Trash2 class="w-4 h-4" />
                  </el-button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 添加账号弹窗 -->
    <el-dialog
      v-model="showAddDialog"
      :title="t('settings.addAccount')"
      :width="'460px'"
      :close-on-click-modal="false"
      :close-on-press-escape="true"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('settings.repositoryUrl') }}</label>
          <el-input v-model="addForm.url" placeholder="https://svn.example.com/repo" size="default" />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.username') }}</label>
          <el-input v-model="addForm.username" size="default" />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.password') }}</label>
          <el-input v-model="addForm.password" type="password" show-password size="default" />
        </div>
      </div>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="showAddDialog = false">{{ t('common.cancel') }}</el-button>
          <el-button type="primary" size="default" :loading="isAdding" :disabled="!addForm.url || !addForm.username || !addForm.password" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleAddAccount">{{ t('common.save') }}</el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 修改密码弹窗 -->
    <el-dialog
      v-model="showEditDialog"
      :title="t('settings.modifyPassword')"
      :width="'400px'"
      :close-on-click-modal="false"
      :close-on-press-escape="true"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('auth.username') }}</label>
          <el-input v-model="editForm.username" disabled size="default" />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">{{ t('settings.newPassword') }}</label>
          <el-input v-model="editForm.newPassword" type="password" show-password size="default" />
        </div>
      </div>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="showEditDialog = false">{{ t('common.cancel') }}</el-button>
          <el-button type="primary" size="default" :loading="isEditing" :disabled="!editForm.newPassword" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleEditPassword">{{ t('common.confirm') }}</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>
