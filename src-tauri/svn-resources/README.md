# SVN 资源（构建副本）

此目录是 `resources/svn/` 的构建副本。Tauri 的 `bundle.resources` 配置不支持 `../` 跨目录引用 glob，因此需要将文件复制到 `src-tauri/` 内部。

**同步规则：** 更新 `resources/svn/` 中的二进制文件时，需同步更新此目录。

- macOS: `resources/svn/macos/svn` → `svn-resources/macos/bin/svn`
- macOS 依赖库: `resources/svn/macos/lib/*.dylib` → `svn-resources/macos/lib/*.dylib`
