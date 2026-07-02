# SVN 平台二进制文件

## macOS ✅ 已部署
- 位置：`resources/svn/macos/`（源）和 `src-tauri/svn-resources/macos/`（构建副本）
- 版本：Subversion 1.14.5 (r1922182)，arm64
- 来源：Homebrew（Apache 2.0 许可证）
- 状态：已通过 `install_name_tool` 修正为自包含包，可在构建时打包

## Windows ❌ 待部署
- 位置：`resources/svn/windows/`
- 当前状态：需要提供 `svn.exe`
- 建议来源：Apache Subversion 官方 Windows 二进制（非 SlikSVN）
- 部署后需在 `src-tauri/tauri.conf.json` 的 `bundle.resources` 中添加条目

## Linux ❌ 待部署
- 位置：`resources/svn/linux/`
- 当前状态：需要提供 `svn`
- 建议来源：发行版包管理器或静态编译
- 部署后需在 `src-tauri/tauri.conf.json` 的 `bundle.resources` 中添加条目
