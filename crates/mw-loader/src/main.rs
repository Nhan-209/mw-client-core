use anyhow::{anyhow, Result};
use std::env;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows_sys::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, WaitForSingleObject, INFINITE, PROCESS_ALL_ACCESS,
};

const TARGET_PROCESS_NAMES: &[&str] = &[
    "MiniWorld.exe",
    "MiniWorld_OverSeas.exe",
    "MiniWorldGame.exe",
    "Client.exe",
];

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("========================================================");
    println!("     Mini World Custom Client - Native DLL Injector     ");
    println!("========================================================");

    let dll_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mw_core.dll"));

    let abs_dll_path = if dll_path.is_absolute() {
        dll_path
    } else {
        env::current_dir()?.join(dll_path)
    };

    if !abs_dll_path.exists() {
        return Err(anyhow!(
            "Target DLL not found at: {:?}. Build it first or provide path.",
            abs_dll_path
        ));
    }

    println!("[*] Injecting DLL: {:?}", abs_dll_path);
    println!("[*] Searching for active Mini World process...");

    let pid = find_target_process()?;
    println!("[+] Found target process PID: {}", pid);

    inject_dll(pid, &abs_dll_path)?;
    println!("[+] Successfully injected mw_core.dll into game!");

    Ok(())
}

fn find_target_process() -> Result<u32> {
    unsafe {
        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(anyhow!("Failed to create toolhelp snapshot"));
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..std::mem::zeroed()
        };

        if Process32FirstW(snapshot, &mut entry) == FALSE {
            CloseHandle(snapshot);
            return Err(anyhow!("Failed to read process entry"));
        }

        loop {
            let mut len = 0;
            while len < entry.szExeFile.len() && entry.szExeFile[len] != 0 {
                len += 1;
            }
            let exe_name = String::from_utf16_lossy(&entry.szExeFile[..len]);

            for target in TARGET_PROCESS_NAMES {
                if exe_name.eq_ignore_ascii_case(target) {
                    CloseHandle(snapshot);
                    return Ok(entry.th32ProcessID);
                }
            }

            if Process32NextW(snapshot, &mut entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot);
    }

    Err(anyhow!(
        "Mini World process not found. Please start the game first!"
    ))
}

fn inject_dll(pid: u32, dll_path: &Path) -> Result<()> {
    let wide_path: Vec<u16> = dll_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let path_bytes_size = wide_path.len() * std::mem::size_of::<u16>();

    unsafe {
        let h_proc = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
        if h_proc == 0 || h_proc == INVALID_HANDLE_VALUE {
            return Err(anyhow!("Failed to OpenProcess for PID: {}", pid));
        }

        let remote_buf = VirtualAllocEx(
            h_proc,
            std::ptr::null_mut(),
            path_bytes_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if remote_buf.is_null() {
            CloseHandle(h_proc);
            return Err(anyhow!("Failed VirtualAllocEx in target process"));
        }

        let mut bytes_written = 0;
        let write_ok = WriteProcessMemory(
            h_proc,
            remote_buf,
            wide_path.as_ptr() as *const c_void,
            path_bytes_size,
            &mut bytes_written,
        );
        if write_ok == FALSE || bytes_written != path_bytes_size {
            VirtualFreeEx(h_proc, remote_buf, 0, MEM_RELEASE);
            CloseHandle(h_proc);
            return Err(anyhow!("Failed WriteProcessMemory in target process"));
        }

        let kernel32 = b"kernel32.dll\0";
        let loadlib = b"LoadLibraryW\0";
        let h_kernel = GetModuleHandleA(kernel32.as_ptr());
        let p_loadlib = GetProcAddress(h_kernel, loadlib.as_ptr());
        if p_loadlib.is_none() {
            VirtualFreeEx(h_proc, remote_buf, 0, MEM_RELEASE);
            CloseHandle(h_proc);
            return Err(anyhow!("Failed to find LoadLibraryW address"));
        }

        let thread_fn = std::mem::transmute(p_loadlib);
        let h_thread = CreateRemoteThread(
            h_proc,
            std::ptr::null(),
            0,
            thread_fn,
            remote_buf,
            0,
            std::ptr::null_mut(),
        );

        if h_thread == 0 || h_thread == INVALID_HANDLE_VALUE {
            VirtualFreeEx(h_proc, remote_buf, 0, MEM_RELEASE);
            CloseHandle(h_proc);
            return Err(anyhow!("CreateRemoteThread failed"));
        }

        WaitForSingleObject(h_thread, INFINITE);

        CloseHandle(h_thread);
        VirtualFreeEx(h_proc, remote_buf, 0, MEM_RELEASE);
        CloseHandle(h_proc);
    }

    Ok(())
}
