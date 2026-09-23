use anyhow::{anyhow, Result};
use log::{error, info};
use minhook_sys::{
    MH_CreateHook, MH_EnableHook, MH_Initialize, MH_ERROR_ALREADY_INITIALIZED, MH_OK,
};
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
    trampoline: LuaLoadBufferFn,
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

        type FarProc = Option<unsafe extern "system" fn() -> isize>;
        let target_ptr = unsafe { std::mem::transmute::<FarProc, *mut c_void>(p_proc) };
        let detour_ptr = hooked_lua_loadbuffer as *const () as *mut c_void;
        let mut orig_ptr: *mut c_void = std::ptr::null_mut();

        unsafe {
            let init_status = MH_Initialize();
            if init_status != MH_OK && init_status != MH_ERROR_ALREADY_INITIALIZED {
                return Err(anyhow!("MH_Initialize failed: {}", init_status));
            }
            let create_status = MH_CreateHook(target_ptr, detour_ptr, &mut orig_ptr);
            if create_status != MH_OK {
                return Err(anyhow!("MH_CreateHook failed: {}", create_status));
            }
            let enable_status = MH_EnableHook(target_ptr);
            if enable_status != MH_OK {
                return Err(anyhow!("MH_EnableHook failed: {}", enable_status));
            }
        }

        let trampoline = unsafe { std::mem::transmute::<*mut c_void, LuaLoadBufferFn>(orig_ptr) };

        let mut lock = HOOK_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire hook state lock"))?;
        *lock = Some(LuaHookState {
            trampoline,
            pending_scripts: Vec::new(),
        });

        info!("[LuaHook] Successfully installed detour on luaL_loadbuffer");
        Ok(())
    }

    /// Thêm script Lua để chuẩn bị thực thi tự động khi máy ảo Lua nạp file
    pub fn queue_script(script: String) {
        let Ok(mut lock) = HOOK_STATE.lock() else {
            return;
        };
        if let Some(state) = lock.as_mut() {
            state.pending_scripts.push(script);
        }
    }
}

fn drain_queued_scripts() {
    let Ok(mut guard) = HOOK_STATE.lock() else {
        return;
    };
    let Some(state_ref) = guard.as_mut() else {
        return;
    };
    for pending in state_ref.pending_scripts.drain(..) {
        info!("[LuaHook] Queued script ready ({} bytes)", pending.len());
    }
}

/// Hàm Detour chặn luaL_loadbuffer
unsafe extern "C" fn hooked_lua_loadbuffer(
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

    if script_name.contains("MainV4LobbyView")
        || script_name.contains("MainLobbyMgr")
        || script_name.contains("minilobby")
    {
        info!("[LuaHook] Intercepted lobby script load: {}", script_name);
        drain_queued_scripts();
    }

    if let Ok(guard) = HOOK_STATE.lock() {
        if let Some(state_ref) = guard.as_ref() {
            return (state_ref.trampoline)(state, buff, size, name);
        }
    }

    error!("[LuaHook] Detour not available!");
    -1
}
