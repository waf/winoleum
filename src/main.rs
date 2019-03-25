extern crate winapi;
extern crate ole32;
extern crate libloading;

use std::io;
use std::io::Error;
use std::ptr;
use std::fs;
use libloading::{Library, Symbol}; // TODO can we replace with winapi https://docs.rs/winapi/*/x86_64-pc-windows-msvc/winapi/um/libloaderapi/fn.GetProcAddress.html
use std::ffi::CString;
use winapi::um::winuser::{SetWindowsHookExA, UnhookWindowsHookEx,
                          RegisterHotKey, MOD_CONTROL, MOD_WIN, MOD_ALT, VK_RIGHT, GetMessageW, MSG, TranslateMessage, DispatchMessageW, WM_HOTKEY,
                          BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetForegroundWindow, HWND_TOP
                          };
use winapi::um::libloaderapi::{ GetModuleHandleA };
use winapi::RIDL;
use winapi::ctypes::{c_int};
use winapi::DEFINE_GUID;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::winerror::{HRESULT};
use winapi::shared::windef::{HWND};
use winapi::shared::guiddef::{REFGUID, GUID};
use winapi::um::servprov::IServiceProvider;
use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx};
use winapi::shared::wtypesbase::CLSCTX_VALID_MASK;
use winapi::Interface;
use winapi::um::objbase::COINIT_MULTITHREADED;

const CLSCTX_LOCAL_SERVER: DWORD = 4;
const WH_CBT: i32 = 5;
const WH_SHELL: i32 = 10;
const WH_KEYBOARD_LL: i32 = 13;

DEFINE_GUID!{CLSID_VirtualDesktopManagerInternal, 0xC5E0CDCA, 0x7B6E, 0x41B2, 0x9F, 0xC4, 0xD9, 0x39, 0x75, 0xCC, 0x46, 0x7B}
DEFINE_GUID!{IID_IVirtualDesktopManagerInternal, 0xAF8DA486, 0x95BB, 0x4460, 0xB3, 0xB7, 0x6E, 0x7A, 0x6B, 0x29, 0x62, 0xB5}
DEFINE_GUID!{IID_IVirtualDesktop, 0xFF72FFDD, 0xBE7E, 0x43FC, 0x9C, 0x03, 0xAD, 0x81, 0x68, 0x1E, 0x88, 0xE4}

const HOTKEY_GO: usize = 1;

fn main() {
}
// https://blogs.msdn.microsoft.com/winsdk/2015/09/10/virtual-desktop-switching-in-windows-10/
// https://github.com/retep998/winapi-rs/blob/a6dd5510212a2f0f5aae66e60949bd6fdaa0a44d/src/um/shobjidl_core.rs
// https://docs.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nf-shobjidl_core-ivirtualdesktopmanager-movewindowtodesktop

fn resize_multiple_windows() {
    unsafe {
        std::thread::sleep_ms(5000);
        let fg = GetForegroundWindow();
        let update = BeginDeferWindowPos(1);
        let result = DeferWindowPos(update, fg, HWND_TOP, 200, 200, 200, 200, 0);
        EndDeferWindowPos(result);
    }
}

fn virtual_desktop() {
    unsafe {
        CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED);

        println!("a");
        let mut service_provider: *mut IServiceProvider = ptr::null_mut();
        let a = CoCreateInstance(&CLSID_ImmersiveShell, ::std::ptr::null_mut(),
            CLSCTX_LOCAL_SERVER, &IServiceProvider::uuidof(), &mut service_provider as *mut _ as *mut _);
        println!("b");


        let mut virtual_desktop_manager: *mut IVirtualDesktopManager = ptr::null_mut();
        (*service_provider).QueryService(&IVirtualDesktopManager::uuidof(), &IVirtualDesktopManager::uuidof(), &mut virtual_desktop_manager as *mut _ as *mut _);
        println!("c");

        let mut on_desktop = std::mem::uninitialized();
        //let mut on_desktop: *mut BOOL = ptr::null_mut();
        (*virtual_desktop_manager).IsWindowOnCurrentVirtualDesktop(ptr::null_mut(), &mut on_desktop);
        println!("d");
        println!("{}", on_desktop);
        (*service_provider).Release();
        println!("e");
    }
}

fn global_hotkey() {
    unsafe {
        println!("registering hotkey");
        RegisterHotKey(ptr::null_mut(), HOTKEY_GO as i32, (MOD_CONTROL | MOD_ALT | MOD_WIN) as u32, VK_RIGHT as u32);

        println!("starting message pump");
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            println!("got message");
            if msg.message == WM_HOTKEY {
                if msg.wParam == HOTKEY_GO {
                    println!("Received GO!");
                } else {
                    println!("Unknown hotkey!");
                }
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn load_dll() {
    println!("Loading library");
    let lib = Library::new("win10wmdll.dll").unwrap();
    unsafe {
        println!("Finding function");
        let hook_callback: Symbol<extern "system" fn(i32, usize, isize) -> isize> = lib.get(b"hook_callback").unwrap();
        println!("Creating hook");
        let module = GetModuleHandleA(CString::new("win10wmdll.dll").unwrap().as_ptr());
        
        let hook_id =
            SetWindowsHookExA(WH_SHELL, Some(*hook_callback), module, 0);

        println!("Error is {:?}", Error::last_os_error());
        println!("Created hook with id {:?}", hook_id);
        
        fs::write("C:/Projects/win10wm/output.txt", "starting").expect("Unable to write file");
        

        let buffer = &mut String::new();
        let input = io::stdin().read_line(buffer);

        // Don't forget to release the hook eventually
        UnhookWindowsHookEx(hook_id);
    }
}

RIDL!{#[uuid(0xa5cd92ff, 0x29be, 0x454c, 0x8d, 0x04, 0xd8, 0x28, 0x79, 0xfb, 0x3f, 0x1b)]
interface IVirtualDesktopManager(IVirtualDesktopManagerVtbl): IUnknown(IUnknownVtbl) {
    fn IsWindowOnCurrentVirtualDesktop(
        topLevelWindow: HWND,
        onCurrentDesktop: *mut BOOL,
    ) -> HRESULT,
    fn GetWindowDesktopId(
        topLevelWindow: HWND,
        desktopId: *mut GUID,
    ) -> HRESULT,
    fn MoveWindowToDesktop(
        topLevelWindow: HWND,
        desktopId: REFGUID,
    ) -> HRESULT,
}}

DEFINE_GUID!{CLSID_ImmersiveShell, 0xC2F03A33, 0x21F5, 0x47FA, 0xB4, 0xBB, 0x15, 0x63, 0x62, 0xA2, 0xF2, 0x39}
DEFINE_GUID!{IID_IServiceProvider10, 0x5140C1, 0x7436, 0x11CE, 0x80, 0x34, 0x00, 0xAA, 0x00, 0x60, 0x09, 0xFA}

DEFINE_GUID!{CLSID_VirtualDesktopManager, 0xAA509086, 0x5CA9, 0x4C25, 0x8F, 0x95, 0x58, 0x9D, 0x3C, 0x07, 0xB4, 0x8A}
DEFINE_GUID!{IID_IVirtualDesktopManager, 0xa5cd92ff, 0x29be, 0x454c, 0x8d, 0x04, 0xd8, 0x28, 0x79, 0xfb, 0x3f, 0x1b}


RIDL!{#[uuid(0xaa509086, 0x5ca9, 0x4c25, 0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a)]
class CVirtualDesktopManager;}