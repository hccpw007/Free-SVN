/** 应用设置（与 Rust Settings 一一对应，含 fallbackToBuiltin） */
export interface AppSettings {
  defaultCheckoutDir: string
  globalIgnorePattern: string
  diffTool: string
  mergeTool: string
  diffCommandTemplate: string
  mergeCommandTemplate: string
  /** 自定义外部工具失败时回退到内置工具（默认 true） */
  fallbackToBuiltin: boolean
  language: string
  autoStart: boolean
  darkMode: boolean
}

export const DEFAULT_SETTINGS: AppSettings = {
  defaultCheckoutDir: '',
  globalIgnorePattern: `*.o
*.lo
*.la
*.al
.libs
*.so
*.so.*
*.a
*.pyc
*.pyo
__pycache__
*.rej
*~
~$*
#*#
.#*
.*.swp
.DS_Store
Thumbs.db
node_modules/
dist/
build/
target/
.idea/
.vscode/
*.class
*.jar
*.war
*.log
*.tmp
*.bak
*.swp
*.swo`,
  diffTool: 'builtin',
  mergeTool: 'builtin',
  diffCommandTemplate: '',
  mergeCommandTemplate: '',
  fallbackToBuiltin: true,
  language: 'system',
  autoStart: false,
  darkMode: false,
}
