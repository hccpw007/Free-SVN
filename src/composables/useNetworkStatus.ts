/**
 * useNetworkStatus - 检测网络可达性，更新 workspaceStore.isOffline
 *
 * 网络检测通过 services/svn.ts 的 checkNetwork 方法调用后端 check_network 命令，
 * 不直接 import { invoke }，确保前端唯一 invoke 入口规则。
 *
 * 使用示例：
 * const { checkNetwork } = useNetworkStatus()
 * const online = await checkNetwork()
 */
import { useWorkspaceStore } from '@/stores/workspace'
import { checkNetwork as svnCheckNetwork } from '@/services/svn'

export function useNetworkStatus() {
  const workspaceStore = useWorkspaceStore()

  async function check(): Promise<boolean> {
    const ok = await svnCheckNetwork()
    workspaceStore.isOffline = !ok
    return ok
  }

  return { checkNetwork: check }
}
