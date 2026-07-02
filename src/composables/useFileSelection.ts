/**
 * useFileSelection - Shift+Click 范围选择 + Ctrl/Cmd+Click 切换选择
 *
 * 使用示例：
 * const { handleClick, reset } = useFileSelection()
 * // 在模板中: @click="handleClick(index, $event, files, selectedPaths, (s) => selectedPaths = s)"
 */
import { ref } from 'vue'

export function useFileSelection() {
  const lastIdx = ref(-1)

  function handleClick(
    idx: number,
    e: MouseEvent,
    items: { path: string }[],
    selected: Set<string>,
    onChg: (s: Set<string>) => void,
  ) {
    const s = new Set(selected)
    if (e.shiftKey && lastIdx.value >= 0) {
      // 范围选择
      for (let i = Math.min(lastIdx.value, idx); i <= Math.max(lastIdx.value, idx); i++) {
        s.add(items[i].path)
      }
    } else if (e.ctrlKey || e.metaKey) {
      // Ctrl/Cmd+Click 切换
      s.has(items[idx].path) ? s.delete(items[idx].path) : s.add(items[idx].path)
    } else {
      // 普通点击：单选
      s.clear()
      s.add(items[idx].path)
    }
    lastIdx.value = idx
    onChg(s)
  }

  function reset() {
    lastIdx.value = -1
  }

  return { handleClick, reset }
}
