extern crate winapi;
extern crate ole32;
extern crate libloading;

use std::io;
use std::io::Error;
use std::io::prelude::*;
use std::ptr;
use std::fs;
use libloading::{Library, Symbol}; // TODO can we replace with winapi https://docs.rs/winapi/*/x86_64-pc-windows-msvc/winapi/um/libloaderapi/fn.GetProcAddress.html
use std::ffi::CString;
use winapi::um::winuser::{SetWindowsHookExA, UnhookWindowsHookEx, CallNextHookEx, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
                          SetWinEventHook, UnhookWinEvent,
                          RegisterHotKey, MOD_CONTROL, MOD_WIN, MOD_ALT, VK_RIGHT, GetMessageW, MSG, TranslateMessage, DispatchMessageW, WM_HOTKEY,
                          BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetForegroundWindow, HWND_TOP, DefWindowProcW,
                          WM_DESTROY, PostQuitMessage, LoadIconW, LoadCursorW, WNDCLASSW, IDI_APPLICATION, RegisterClassW, CreateWindowExW,
                          WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, WM_USER, CreatePopupMenu, MENUINFO, MIM_STYLE, MIM_APPLYTOSUBMENUS, MNS_NOTIFYBYPOS,
                          SetMenuInfo, RegisterShellHookWindow, RegisterWindowMessageW, HSHELL_WINDOWCREATED, HSHELL_WINDOWDESTROYED,
                          EnumWindows, WNDENUMPROC, IsWindowVisible, GetWindowTextA, GetWindow, GW_OWNER, GetWindowRect, GetWindowLongW, GWL_EXSTYLE,
                          WS_EX_TOOLWINDOW, GetSystemMetrics, SM_CXMAXIMIZED, SM_CYMAXIMIZED, SystemParametersInfoA, SPI_GETWORKAREA, ShowWindow, IsZoomed,
                          SW_SHOWNORMAL, SWP_DRAWFRAME, SetWindowPos, WH_CALLWNDPROC,
                          GetSysColor, COLOR_ACTIVEBORDER, SystemParametersInfoW, SPI_GETNONCLIENTMETRICS, NONCLIENTMETRICSW
                          };
use winapi::um::libloaderapi::{ GetModuleHandleA };
use winapi::{RIDL};
use winapi::ctypes::{c_int};
use winapi::DEFINE_GUID;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
use winapi::shared::minwindef::{BOOL, DWORD, LPVOID};
use winapi::shared::winerror::{HRESULT};
use winapi::shared::windef::{HWND, HWINEVENTHOOK, HBRUSH, HMENU, HICON, RECT};
use winapi::shared::guiddef::{REFGUID, GUID};
use winapi::um::winnt::{LPCWSTR};
use winapi::um::servprov::IServiceProvider;
use winapi::um::shellapi::{NOTIFYICONDATAW, NOTIFYICONDATAW_u, NIM_ADD, NIM_MODIFY, NIM_DELETE, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP};
use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx};
use winapi::shared::wtypesbase::CLSCTX_VALID_MASK;
use winapi::Interface;
use winapi::um::objbase::COINIT_MULTITHREADED;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, HINSTANCE, TRUE, FALSE, PBYTE};
use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use stretch::{style::*, node::Node, geometry::Size};

const CLSCTX_LOCAL_SERVER: DWORD = 4;
const WH_CBT: i32 = 5;
const WH_SHELL: i32 = 10;
const WH_KEYBOARD_LL: i32 = 13;

DEFINE_GUID!{CLSID_VirtualDesktopManagerInternal, 0xC5E0CDCA, 0x7B6E, 0x41B2, 0x9F, 0xC4, 0xD9, 0x39, 0x75, 0xCC, 0x46, 0x7B}
DEFINE_GUID!{IID_IVirtualDesktopManagerInternal, 0xAF8DA486, 0x95BB, 0x4460, 0xB3, 0xB7, 0x6E, 0x7A, 0x6B, 0x29, 0x62, 0xB5}
DEFINE_GUID!{IID_IVirtualDesktop, 0xFF72FFDD, 0xBE7E, 0x43FC, 0x9C, 0x03, 0xAD, 0x81, 0x68, 0x1E, 0x88, 0xE4}

const HOTKEY_GO: usize = 1;

#[derive(Debug)]
struct CbWindowInfo {
    title: String,
    hwnd: HWND,
}
#[derive(Debug)]
struct CallbackState {
    windows: Vec<CbWindowInfo>,
}

#[derive(Debug)]
struct Window {
    title: String,
    hwnd: HWND,
    layout_node: Node,
}

fn main() {
    unsafe{
        let color = GetSysColor(COLOR_ACTIVEBORDER);
        println!("current color is {}", color); // 11842740
        let mut metrics = std::mem::zeroed::<NONCLIENTMETRICSW>();
        let size = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
        SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size, &mut metrics as *mut _ as *mut _, 0);
        println!("current border is {}", metrics.iBorderWidth); //0
        println!("current padded border is {}", metrics.iPaddedBorderWidth); //0

        return;
        let mut work_area : RECT = std::mem::zeroed();
        SystemParametersInfoA(SPI_GETWORKAREA, 0, &mut work_area as *mut RECT as LPVOID, 0);

        let view_width = work_area.right - work_area.left;
        let view_height = work_area.bottom - work_area.top;
        println!("view_width: {}", view_width);
        println!("view_height: {}", view_height);

        let mut cbstate = CallbackState { windows: vec![] };
        EnumWindows(Some(store_window), &mut cbstate as *mut CallbackState as LPARAM);

        let all_windows = cbstate.windows.iter()
            .map(|w| Window {
                hwnd: w.hwnd,
                title: w.title.clone(),
                layout_node: Node::new(Style {
                    size: Size {
                        width: Dimension::Auto,
                        height: Dimension::Auto,
                    },
                    align_self: AlignSelf::Stretch,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Dimension::Auto,
                    ..Default::default()
                }, vec![])
            })
            .collect::<Vec<_>>();

        let horizontal_container = Node::new(Style {
            size: Size { 
                width: Dimension::Points(view_width as f32), 
                height: Dimension::Points(view_height as f32),
            },
            display: Display::Flex,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        }, all_windows.iter().map(|w| &w.layout_node).collect::<Vec<_>>());


        if let Ok(layout) = horizontal_container.compute_layout(Size::undefined()) {
            println!("{:?}", layout);

            let mut update = BeginDeferWindowPos(all_windows.len() as i32);
            for (i, window) in all_windows.iter().enumerate() {
                let pos = &layout.children[i];
                if IsZoomed(window.hwnd) > 0 {
                    // we can't reposition maximized windows, so unmaximize them.
                    ShowWindow(window.hwnd, SW_SHOWNORMAL);
                }
                update = DeferWindowPos(update, window.hwnd, HWND_TOP,
                    work_area.left + pos.location.x as i32,
                    work_area.top + pos.location.y as i32,
                    pos.size.width as i32,
                    pos.size.height as i32,
                    0);
            }
            EndDeferWindowPos(update);
        }

        return;

        let class_name = to_wstring("winoleum");
        let hinstance = GetModuleHandleA(ptr::null_mut());
        let wnd = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: LoadIconW(0 as HINSTANCE, IDI_APPLICATION),
            hCursor: LoadCursorW(0 as HINSTANCE, IDI_APPLICATION),
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: class_name.as_ptr(),
        };
        if RegisterClassW(&wnd) == 0 {
            println!("Error creating window class");
        }
        let hwnd = CreateWindowExW(0,
            class_name.as_ptr(),
            to_wstring("rust_systray_window").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            0,
            CW_USEDEFAULT,
            0,
            0 as HWND,
            0 as HMENU,
            0 as HINSTANCE,
            ptr::null_mut());
        if hwnd == ptr::null_mut() {
            println!("Error creating window class");
        }
        let mut notifyicon = get_nid_struct(&hwnd);
        if Shell_NotifyIconW(NIM_ADD, &mut notifyicon as *mut NOTIFYICONDATAW) == 0 {
            println!("Error adding menu icon");
        }
        let hmenu = CreatePopupMenu();
        let m = MENUINFO {
            cbSize: std::mem::size_of::<MENUINFO>() as DWORD,
            fMask: MIM_APPLYTOSUBMENUS | MIM_STYLE,
            dwStyle: MNS_NOTIFYBYPOS,
            cyMax: 0 as UINT,
            hbrBack: 0 as HBRUSH,
            dwContextHelpID: 0 as DWORD,
            dwMenuData: 0
        };
        if SetMenuInfo(hmenu, &m as *const MENUINFO) == 0 {
            println!("Error setting up menu");
        }

        RegisterWindowMessageW(to_wstring("SHELLHOOK").as_ptr());
        println!("Error is {:?}", Error::last_os_error());
        RegisterShellHookWindow(hwnd);
        println!("Error is {:?}", Error::last_os_error());


        println!("starting message pump");
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            println!("got message");
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // pause
        println!("pausing");
        let buffer = &mut String::new();
        let input = io::stdin().read_line(buffer);

        if Shell_NotifyIconW(NIM_DELETE, &mut notifyicon as *mut NOTIFYICONDATAW) == 0 {
            println!("Error removing menu icon");
        }
    }
}

fn get_nid_struct(hwnd : &HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
        hWnd: *hwnd,
        uID: 0x1 as UINT,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_USER + 1,
        hIcon: unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) },
        szTip: create_tooltip("Winoleum"),
        dwState: 0 as DWORD,
        dwStateMask: 0 as DWORD,
        szInfo: [0 as u16; 256],
        u: unsafe { std::mem::zeroed() },
        szInfoTitle: [0 as u16; 64],
        dwInfoFlags: 0 as UINT,
        guidItem: GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8]
        },
        hBalloonIcon: 0 as HICON
    }
}

fn to_wstring(str : &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}

fn create_tooltip(str : &str) -> [u16; 128] {
    let mut trayToolTipInt = [0; 128];
    let trayToolTipUTF16 = to_wstring(str);
    trayToolTipInt[..trayToolTipUTF16.len()].copy_from_slice(&trayToolTipUTF16);
    trayToolTipInt
}

unsafe extern "system" fn window_proc(h_wnd :HWND,
	                                    msg :UINT,
                                      w_param :WPARAM,
                                      l_param :LPARAM) -> LRESULT
{
    let event_type = w_param as i32;
    let window_handle = l_param as HWND;

    if msg == WM_DESTROY {
        PostQuitMessage(0);
    } else if event_type == HSHELL_WINDOWCREATED {
        println!("window created {}", l_param);

        let update = BeginDeferWindowPos(1);
        let result = DeferWindowPos(update, window_handle, HWND_TOP, 200, 200, 400, 400, 0);
        EndDeferWindowPos(result);
    } else if w_param as i32 == HSHELL_WINDOWDESTROYED {
        println!("window destroyed {}", l_param);
    }
    return DefWindowProcW(h_wnd, msg, w_param, l_param);
}

unsafe extern "system" fn store_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let cbs = &mut *(lparam as *mut CallbackState);

    if IsWindowVisible(hwnd) == 0 {
        return TRUE;
    }
    let title = get_window_title(hwnd);
    
    if title.len() == 0 {
        return TRUE
    }

    // filter out suspended windows store apps
    let mut is_cloaked = FALSE;
    DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut is_cloaked as *mut BOOL as LPVOID, std::mem::size_of::<BOOL>() as DWORD);
    if is_cloaked != 0 {
        return TRUE;
    }

    // Tool windows should not be displayed either, these do not appear in the task bar.
    if GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_TOOLWINDOW as i32 == WS_EX_TOOLWINDOW as i32 {
        return TRUE;
    }

    cbs.windows.push(CbWindowInfo { title, hwnd });

    TRUE
}
fn get_window_title(hwnd: HWND) -> String {
    let size = 256;
    let mut buf = Vec::with_capacity(size as usize);
    unsafe {
        let winstr = GetWindowTextA(hwnd, buf.as_mut_ptr(), size);
        buf.set_len(winstr as usize);
    }
    let windowtitle = unsafe {
        let mut bufchars = std::mem::transmute::<Vec<i8>, Vec<u8>>(buf);
        bufchars.truncate(bufchars.len());
        let windowtitle = {String::from_utf8_lossy(&bufchars.clone()).into_owned()};
        windowtitle
    };
    windowtitle
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


#[no_mangle]
pub extern "system" fn window_create_callback(code: i32, wParam: usize, lParam: isize) -> isize {
    println!("Window Opened");

    unsafe {

        let mut file = fs::OpenOptions::new()
            .append(true)
            .open("C:/Projects/winoleum/output.txt")
            .unwrap();
        writeln!(file, "monitor triggered").unwrap();
        return CallNextHookEx(ptr::null_mut(), code, wParam, lParam);
    }
}

fn monitor_windows() {
    unsafe {


        //let hook_id = SetWinEventHook(EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, ptr::null_mut(), Some(monitor_window), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);

        let hook_id =
            SetWindowsHookExA(WH_SHELL, Some(window_create_callback), ptr::null_mut(), 0);
        println!("Error is {:?}", Error::last_os_error());

        let buffer = &mut String::new();
        let input = io::stdin().read_line(buffer);

        // Don't forget to release the hook eventually
        UnhookWindowsHookEx(hook_id);
    }
}

fn load_dll() {
    println!("Loading library");
    let lib = Library::new("winoleumdll3.dll").unwrap();
    unsafe {
        println!("Finding function");
        let hook_callback: Symbol<extern "system" fn(i32, usize, isize) -> isize> = lib.get(b"hook_callback").unwrap();
        println!("Creating hook");
        let module = GetModuleHandleA(CString::new("winoleumdll3.dll").unwrap().as_ptr());
        
        let hook_id =
            SetWindowsHookExA(WH_CALLWNDPROC, Some(*hook_callback), module, 0);

        println!("Error is {:?}", Error::last_os_error());
        println!("Created hook with id {:?}", hook_id);
        
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