use anyhow::{anyhow, Result};
use log::info;
use minhook_sys::{
    MH_CreateHook, MH_EnableHook, MH_Initialize, MH_ERROR_ALREADY_INITIALIZED, MH_OK,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type CreateFileWFn = unsafe extern "system" fn(
    lp_file_name: *const u16,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: *const SECURITY_ATTRIBUTES,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: HANDLE,
) -> HANDLE;

static VFS_STATE: Mutex<Option<VfsState>> = Mutex::new(None);

struct VfsState {
    trampoline: CreateFileWFn,
    /// Ánh xạ: tên tệp gốc (chữ thường) -> đường dẫn tệp thay thế trên đĩa
    redirections: HashMap<String, PathBuf>,
}

pub struct VfsHookManager;

impl VfsHookManager {
    /// Cài đặt VFS Hook chặn CreateFileW của hệ điều hành
    pub fn install() -> Result<()> {
        let kernel32 = b"kernel32.dll\0";
        let proc_name = b"CreateFileW\0";

        let h_module = unsafe { GetModuleHandleA(kernel32.as_ptr()) };
        if h_module == 0 {
            return Err(anyhow!("Cannot get kernel32.dll handle"));
        }

        let p_proc = unsafe { GetProcAddress(h_module, proc_name.as_ptr()) };
        if p_proc.is_none() {
            return Err(anyhow!("Cannot resolve CreateFileW in kernel32.dll"));
        }

        type FarProc = Option<unsafe extern "system" fn() -> isize>;
        let target_ptr = unsafe { std::mem::transmute::<FarProc, *mut c_void>(p_proc) };
        let detour_ptr = hooked_create_file_w as *const () as *mut c_void;
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

        let trampoline = unsafe { std::mem::transmute::<*mut c_void, CreateFileWFn>(orig_ptr) };

        let mut lock = VFS_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire VFS lock"))?;
        *lock = Some(VfsState {
            trampoline,
            redirections: HashMap::new(),
        });

        info!("[VFSHook] Installed transparent file redirection hook");
        Ok(())
    }

    /// Đăng ký chuyển hướng một tệp tài nguyên sang tệp custom
    pub fn register_redirection(original_file_name: &str, custom_path: PathBuf) {
        let Ok(mut lock) = VFS_STATE.lock() else {
            return;
        };
        let Some(state) = lock.as_mut() else {
            return;
        };
        let key = original_file_name.to_lowercase();
        info!(
            "[VFSHook] Registered redirect: {} -> {:?}",
            key, custom_path
        );
        state.redirections.insert(key, custom_path);
    }

    /// Tự động quét thư mục custom_assets/textures/ để ánh xạ ảnh nền sảnh
    pub fn scan_custom_folder(dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let name_str = file_name.to_string();
                Self::register_redirection(&name_str, path);
            }
        }
    }
}

fn resolve_redirect_path(path_str: &str) -> Option<PathBuf> {
    let guard = VFS_STATE.lock().ok()?;
    let state = guard.as_ref()?;
    let lower = path_str.to_lowercase();
    for (orig_name, target) in &state.redirections {
        if lower.ends_with(orig_name) {
            return Some(target.clone());
        }
    }
    None
}

unsafe extern "system" fn hooked_create_file_w(
    lp_file_name: *const u16,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: *const SECURITY_ATTRIBUTES,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: HANDLE,
) -> HANDLE {
    if lp_file_name.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    let mut len = 0;
    while *lp_file_name.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(lp_file_name, len);
    let path_str = String::from_utf16_lossy(slice);

    let redirect_path = resolve_redirect_path(&path_str);

    let target_wide: Vec<u16>;
    let final_ptr = if let Some(target) = redirect_path {
        info!("[VFSHook] Redirected file: {} -> {:?}", path_str, target);
        target_wide = target
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        target_wide.as_ptr()
    } else {
        lp_file_name
    };

    if let Ok(guard) = VFS_STATE.lock() {
        if let Some(state_ref) = guard.as_ref() {
            return (state_ref.trampoline)(
                final_ptr,
                dw_desired_access,
                dw_share_mode,
                lp_security_attributes,
                dw_creation_disposition,
                dw_flags_and_attributes,
                h_template_file,
            );
        }
    }

    INVALID_HANDLE_VALUE
}
