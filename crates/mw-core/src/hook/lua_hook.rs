use anyhow::{anyhow, Result};
use log::{error, info};
use retour::GenericDetour;
use std::ffi::{c_char, c_void, CStr};
use std::sync::Mutex;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type LuaLoadBufferFn = unsafe extern "C" fn(
    state: *mut c_void,
    buff: *const u8,
    size: usize,
    name: *const c_char,
) -> i32;

static HOOK_STATE: Mutex<Option<LuaHookState>> = Mutex::new(None);

struct LuaHookState {
    detour: GenericDetour<LuaLoadBufferFn>,
    pending_scripts: Vec<String>,
}

pub struct LuaHookManager;

impl LuaHookManager {
    /// Cài đặt hook vào máy ảo Lua (liblua.dll) của Rainbow Engine
    pub fn install() -> Result<()> {
        let module_name = b"liblua.dll\0";
        let proc_name = b"luaL_loadbuffer\0";

        let h_module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
        if h_module == 0 {
            return Err(anyhow!("liblua.dll is not loaded in current process yet"));
        }

        let p_proc = unsafe { GetProcAddress(h_module, proc_name.as_ptr()) };
        if p_proc.is_none() {
            return Err(anyhow!("Cannot resolve luaL_loadbuffer in liblua.dll"));
        }

        let target_fn: LuaLoadBufferFn = unsafe { std::mem::transmute(p_proc) };
        let detour = unsafe { GenericDetour::new(target_fn, hooked_luaL_loadbuffer)? };

        unsafe {
            detour.enable()?;
        }

        let mut lock = HOOK_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire hook state lock"))?;
        *lock = Some(LuaHookState {
            detour,
            pending_scripts: Vec::new(),
        });

        info!("[LuaHook] Successfully installed detour on luaL_loadbuffer");
        Ok(())
    }

    /// Thêm script Lua để chuẩn bị thực thi tự động khi máy ảo Lua nạp file
    pub fn queue_script(script: String) {
        if let Ok(mut lock) = HOOK_STATE.lock() {
            if let Some(state) = lock.as_mut() {
                state.pending_scripts.push(script);
            }
        }
    }
}

/// Hàm Detour chặn luaL_loadbuffer
unsafe extern "C" fn hooked_luaL_loadbuffer(
    state: *mut c_void,
    buff: *const u8,
    size: usize,
    name: *const c_char,
) -> i32 {
    let script_name = if !name.is_null() {
        CStr::from_ptr(name).to_string_lossy()
    } else {
        "anonymous".into()
    };

    // Kiểm tra và thực thi script can thiệp UI khi các file điều hướng sảnh được nạp
    if script_name.contains("MainV4LobbyView")
        || script_name.contains("MainLobbyMgr")
        || script_name.contains("minilobby")
    {
        info!("[LuaHook] Intercepted lobby script load: {}", script_name);
    }

    // Lấy con trỏ gốc để tiếp tục thực thi
    let original = {
        let lock = HOOK_STATE.lock().ok();
        lock.and_then(|guard| guard.as_ref().map(|s| s.detour.trampoline()))
    };

    if let Some(trampoline) = original {
        trampoline(state, buff, size, name)
    } else {
        error!("[LuaHook] Trampoline not found!");
        -1
    }
}
