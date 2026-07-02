use tauri::AppHandle;

/// 设置系统托盘徽章（操作进行中/完成）
#[tauri::command]
pub async fn set_tray_badge(app: AppHandle, visible: bool) -> Result<(), String> {
    // 通过 app.tray_by_id 获取系统托盘
    if let Some(tray) = app.tray_by_id("main") {
        if visible {
            // 设置操作进行中状态的 tooltip
            let _ = tray.set_tooltip(Some("SVN 操作进行中..."));
        } else {
            // 恢复默认 tooltip（图标保持默认不变）
            let _ = tray.set_tooltip(Some("Free-SVN"));
        }
    }
    Ok(())
}

/// macOS Dock badge 设置（使用 objc crate 直接调用 AppKit API）
///
/// 在 lib.rs 托盘菜单操作（update/cleanup）的 spawn 前后调用。
///
/// # 安全性
/// 1. NSApp 全局指针在应用启动后初始化，setup() 之后调用保证非空
/// 2. dockTile / setBadgeLabel 消息发送不持有返回值 —— 不涉及 Rust 所有权跨语言传递
/// 3. NSString 通过 Class::get 获取，stringWithUTF8String 创建临时的 autoreleased 字符串
/// 4. setBadgeLabel: nil 在 AppKit 中被定义为"清除 badge"
/// 5. 所有 objc 调用在 #[cfg(target_os = "macos")] 保护下仅对 macOS 编译
#[cfg(target_os = "macos")]
pub fn set_dock_badge(visible: bool) {
    use objc::{msg_send, sel, sel_impl};
    use objc::runtime::{Object, Class};
    #[link(name = "AppKit", kind = "framework")]
    extern "C" {
        static NSApp: *mut Object;
    }
    unsafe {
        let dock_tile: *mut Object = msg_send![NSApp, dockTile];
        if visible {
            let cls = Class::get("NSString").unwrap();
            // 使用 CString literal（Rust 1.77+），避免 manual_c_str_literals 警告
            let s: *const i8 = c"\u{25CF}".as_ptr();
            let label: *mut Object = msg_send![cls, stringWithUTF8String: s];
            let _: () = msg_send![dock_tile, setBadgeLabel: label];
        } else {
            let nil_obj: *mut Object = std::ptr::null_mut();
            let _: () = msg_send![dock_tile, setBadgeLabel: nil_obj];
        }
    }
}
