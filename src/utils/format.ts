/**
 * 数字格式化工具函数
 */

/**
 * 格式化数字为人类可读形式。
 *
 * - < 1000: 原样返回（如 999）
 * - >= 1000: 显示为 k（如 1234 → "1.2k"）
 * - >= 1000000: 显示为 M（如 1234567 → "1.2M"）
 */
export function formatNumber(n: number): string {
  if (n >= 1000000) {
    const val = n / 1000000
    return val % 1 === 0 ? val.toFixed(0) + 'M' : val.toFixed(1) + 'M'
  }
  if (n >= 1000) {
    const val = n / 1000
    return val % 1 === 0 ? val.toFixed(0) + 'k' : val.toFixed(1) + 'k'
  }
  return String(n)
}

/**
 * 将秒数格式化为 mm:ss 格式。
 *
 * @param seconds 总秒数（非负整数）
 */
export function formatElapsed(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}
