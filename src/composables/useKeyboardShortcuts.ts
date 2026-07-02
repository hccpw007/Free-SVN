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
// 模块级变量，避免异步导入
let isOpRunning = false

/** 由 App.vue 或 useOperationGuard 调用以更新操作进行中状态 */
export function setOperationRunning(v: boolean) {
  isOpRunning = v
}

function ensureListener() {
  if (globalListener) return
  globalListener = (e: KeyboardEvent) => {
    // isOperationRunning 时仅 Esc 可响应
    if (isOpRunning && e.key !== 'Escape') return

    // 检查是否有输入框聚焦（输入框聚焦时不触发快捷键，避免干扰输入）
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
      // Escape 和 Ctrl+Enter 即使在输入框中也应生效
      if (e.key !== 'Escape' && !(e.ctrlKey && e.key === 'Enter')) return
    }

    // 更精确的快捷键匹配：使用 split('+') 解析，匹配修饰键和主键
    const pressedMods = {
      ctrl: e.ctrlKey,
      shift: e.shiftKey,
      alt: e.altKey,
      meta: e.metaKey,
    }
    const mainKey = e.key.toLowerCase()

    // 按作用域优先级匹配
    for (const scope of ['dialog' as const, 'panel' as const, 'global' as const]) {
      for (const [, def] of registered) {
        if ((def.scope ?? 'global') !== scope) continue

        // 解析注册的快捷键字符串
        const parts = def.keys.toLowerCase().split('+')
        const targetMainKey = parts[parts.length - 1]
        const targetMods = parts.slice(0, -1)

        // 主键必须匹配
        if (mainKey !== targetMainKey) continue

        // 所有指定修饰键必须按下
        const allModsMatch = targetMods.every(m => pressedMods[m as keyof typeof pressedMods])
        if (!allModsMatch) continue

        // 未指定的修饰键不得按下（避免 Ctrl+Shift+W 误触发 Ctrl+W）
        const extraModsPressed = (
          (pressedMods.ctrl && !targetMods.includes('ctrl')) ||
          (pressedMods.shift && !targetMods.includes('shift')) ||
          (pressedMods.alt && !targetMods.includes('alt')) ||
          (pressedMods.meta && !targetMods.includes('meta'))
        )
        if (extraModsPressed) continue

        e.preventDefault()
        e.stopPropagation()
        def.handler()
        return
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
