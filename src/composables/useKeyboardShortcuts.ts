/**
 * useKeyboardShortcuts - 全局键盘快捷键注册与管理
 * 支持作用域管理：对话框 > 面板 > 全局
 * ProgressOverlay 期间仅 Esc 生效
 *
 * 使用示例：
 * const { register, unregister } = useKeyboardShortcuts()
 * onMounted(() => register([{ id: 'close', keys: 'Escape', handler: () => close(), scope: 'panel' }]))
 * onUnmounted(() => unregister('close'))
 */

type ShortcutScope = 'dialog' | 'panel' | 'global'

interface ShortcutDef {
  id: string
  keys: string
  handler: () => void
  scope?: ShortcutScope
}

const registered = new Map<string, ShortcutDef>()
let globalListener: ((e: KeyboardEvent) => void) | null = null

function ensureListener() {
  if (globalListener) return
  globalListener = (e: KeyboardEvent) => {
    // 检查是否有输入框聚焦（输入框聚焦时不触发快捷键，避免干扰输入）
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
      // Escape 和 Ctrl+Enter 即使在输入框中也应生效
      if (e.key !== 'Escape' && !(e.ctrlKey && e.key === 'Enter')) return
    }

    const key = e.key.toLowerCase()
    const ctrl = e.ctrlKey || e.metaKey
    const parts: string[] = []
    if (ctrl) parts.push('ctrl')
    if (e.shiftKey) parts.push('shift')
    if (e.altKey) parts.push('alt')
    parts.push(key)
    const combo = parts.join('+')

    // 按作用域优先级匹配
    for (const scope of ['dialog' as const, 'panel' as const, 'global' as const]) {
      for (const [, def] of registered) {
        if ((def.scope ?? 'global') !== scope) continue
        if (def.keys.toLowerCase() === combo || def.keys.toLowerCase() === key) {
          e.preventDefault()
          e.stopPropagation()
          def.handler()
          return
        }
      }
    }
  }
  window.addEventListener('keydown', globalListener)
}

export function useKeyboardShortcuts() {
  function register(defs: ShortcutDef[]) {
    ensureListener()
    for (const def of defs) {
      registered.set(def.id, def)
    }
  }

  function unregister(id: string) {
    registered.delete(id)
  }

  return { register, unregister }
}
