/**
 * useAutoRefresh - Tauri 窗口焦点事件 → 300ms 防抖 → fileListStore.refresh()
 *
 * 使用 Tauri 2 的 onFocusChanged 事件替代 DOM window.focus，
 * 确保在 Tauri WebView 中窗口从后台切回时能正确触发刷新。
 *
 * 使用示例：
 * useAutoRefresh() // 在 MainLayout 或 HomePage 的 setup 中调用
 */
import { onMounted, onUnmounted } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function useAutoRefresh() {
  const fileListStore = useFileListStore()
  let t: ReturnType<typeof setTimeout> | null = null
  let unlisten: (() => void) | null = null

  const f = () => {
    if (t) clearTimeout(t)
    t = setTimeout(() => fileListStore.refresh(), 300)
  }

  onMounted(async () => {
    // 使用 Tauri 2 的 onFocusChanged 事件
    unlisten = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) f()
    })
  })

  onUnmounted(() => {
    if (unlisten) unlisten()
    if (t) clearTimeout(t)
  })
}
