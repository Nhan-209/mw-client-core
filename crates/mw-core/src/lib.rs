pub mod hook;
pub mod ui_manager;

use hook::{LuaHookManager, VfsHookManager};
use log::{error, info};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use ui_manager::UIManager;
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE, TRUE};
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

/// Luồng khởi tạo chính của Runtime khi được nạp vào game
fn runtime_entrypoint() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    info!("================================================");
    info!("   Mini World Custom Client Runtime (Rust)     ");
    info!(
        "   Version: {}                                ",
        env!("CARGO_PKG_VERSION")
    );
    info!("================================================");

    let ui_mgr = UIManager::new();

    // 1. Kích hoạt Virtual File System Redirection
    if let Err(e) = VfsHookManager::install() {
        error!("[Runtime] Failed to install VFS hook: {:?}", e);
    } else {
        let custom_dir = PathBuf::from("custom_assets/textures");
        VfsHookManager::scan_custom_folder(&custom_dir);
    }

    // 2. Chờ module máy ảo liblua.dll nạp đầy đủ (polling nhẹ không nghẽn)
    thread::spawn(move || {
        let mut retries = 0;
        while retries < 60 {
            if LuaHookManager::install().is_ok() {
                info!("[Runtime] Lua VM hook hooked successfully!");
                let script = ui_mgr.build_injection_script();
                LuaHookManager::queue_script(script);
                break;
            }
            thread::sleep(Duration::from_millis(500));
            retries += 1;
        }
    });
}

/// DllMain: Điểm vào chuẩn Windows khi injector tiêm DLL vào game
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *mut std::ffi::c_void,
) -> BOOL {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // Khởi tạo luồng riêng biệt để tránh deadlock trong Loader Lock
            thread::spawn(runtime_entrypoint);
        }
        DLL_PROCESS_DETACH => {
            info!("[Runtime] Detached from game process.");
        }
        _ => {}
    }
    TRUE
}

/// Hàm export bổ sung cho các injector gọi tường minh
#[no_mangle]
pub extern "C" fn mw_client_init() -> i32 {
    runtime_entrypoint();
    0
}
