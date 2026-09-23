use anyhow::{anyhow, Result};
use log::info;
use retour::GenericDetour;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::CreateFileW;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type CreateFileWFn = unsafe extern "system" fn(
    lpFileName: *const u16,
    dwDesiredAccess: u32,
    dwShareMode: u32,
    lpSecurityAttributes: *const SECURITY_ATTRIBUTES,
    dwCreationDisposition: u32,
    dwFlagsAndAttributes: u32,
    hTemplateFile: HANDLE,
) -> HANDLE;

static VFS_STATE: Mutex<Option<VfsState>> = Mutex::new(None);

struct VfsState {
    detour: GenericDetour<CreateFileWFn>,
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
        if h_module.is_null() {
            return Err(anyhow!("Cannot get kernel32.dll handle"));
        }

        let p_proc = unsafe { GetProcAddress(h_module, proc_name.as_ptr()) };
        if p_proc.is_none() {
            return Err(anyhow!("Cannot resolve CreateFileW in kernel32.dll"));
        }

        let target_fn: CreateFileWFn = unsafe { std::mem::transmute(p_proc) };
        let detour = unsafe { GenericDetour::new(target_fn, hooked_CreateFileW)? };

        unsafe {
            detour.enable()?;
        }

        let mut lock = VFS_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire VFS lock"))?;
        *lock = Some(VfsState {
            detour,
            redirections: HashMap::new(),
        });

        info!("[VFSHook] Installed transparent file redirection hook");
        Ok(())
    }

    /// Đăng ký chuyển hướng một tệp tài nguyên sang tệp custom
    pub fn register_redirection(original_file_name: &str, custom_path: PathBuf) {
        if let Ok(mut lock) = VFS_STATE.lock() {
            if let Some(state) = lock.as_mut() {
                let key = original_file_name.to_lowercase();
                info!(
                    "[VFSHook] Registered redirect: {} -> {:?}",
                    key, custom_path
                );
                state.redirections.insert(key, custom_path);
            }
        }
    }

    /// Tự động quét thư mục custom_assets/textures/ để ánh xạ ảnh nền sảnh
    pub fn scan_custom_folder(dir: &Path) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        Self::register_redirection(file_name, path);
                    }
                }
            }
        }
    }
}

unsafe extern "system" fn hooked_CreateFileW(
    lpFileName: *const u16,
    dwDesiredAccess: u32,
    dwShareMode: u32,
    lpSecurityAttributes: *const SECURITY_ATTRIBUTES,
    dwCreationDisposition: u32,
    dwFlagsAndAttributes: u32,
    hTemplateFile: HANDLE,
) -> HANDLE {
    if lpFileName.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    // Đọc chuỗi wide string
    let mut len = 0;
    while *lpFileName.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(lpFileName, len);
    let path_str = String::from_utf16_lossy(slice);

    // Kiểm tra có nằm trong danh sách chuyển hướng không
    let redirect_path = {
        let lock = VFS_STATE.lock().ok();
        lock.and_then(|guard| {
            guard.as_ref().and_then(|state| {
                let lower = path_str.to_lowercase();
                for (orig_name, target) in &state.redirections {
                    if lower.ends_with(orig_name) {
                        return Some(target.clone());
                    }
                }
                None
            })
        })
    };

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
        lpFileName
    };

    let original = {
        let lock = VFS_STATE.lock().ok();
        lock.and_then(|guard| guard.as_ref().map(|s| s.detour.trampoline()))
    };

    if let Some(trampoline) = original {
        trampoline(
            final_ptr,
            dwDesiredAccess,
            dwShareMode,
            lpSecurityAttributes,
            dwCreationDisposition,
            dwFlagsAndAttributes,
            hTemplateFile,
        )
    } else {
        INVALID_HANDLE_VALUE
    }
}
