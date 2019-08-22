extern crate winapi;

use std::io::prelude::*;
use std::ptr;
use std::fs;
use winapi::um::winuser::{ CallNextHookEx, WM_NCCALCSIZE, WM_NCPAINT, WM_NCACTIVATE, WM_ERASEBKGND, WM_WINDOWPOSCHANGED, HC_ACTION, CWPSTRUCT, GetWindowRect,
    GetWindowDC, FillRect, IntersectRect, OffsetRect, ReleaseDC, RedrawWindow, RDW_UPDATENOW,
    LPNCCALCSIZE_PARAMS, NCCALCSIZE_PARAMS,
    DCX_WINDOW, DCX_CACHE, DCX_INTERSECTRGN, DCX_LOCKWINDOWUPDATE,
    GetDCEx
};
use winapi::um::wingdi::{ Rectangle, CreatePen, PS_INSIDEFRAME, RGN_COPY, CombineRgn, CreateRectRgn, NULLREGION, CreateSolidBrush, DeleteObject, RGB, GetRgnBox, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, DeleteDC };
use winapi::shared::windef::{RECT, HGDIOBJ, HPEN };
use winapi::shared::minwindef::{LPVOID, HRGN, DWORD, LPCVOID};
use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DWMNCRENDERINGPOLICY, DWMNCRP_DISABLED, DWMNCRP_ENABLED, DwmSetWindowAttribute, DWMWA_NCRENDERING_POLICY };
use winapi::um::uxtheme::MARGINS;

#[no_mangle]
pub extern "system" fn hook_callback(code: i32, wParam: usize, lParam: isize) -> isize {
    unsafe {
        
        if code == HC_ACTION as i32 {

            // https://stackoverflow.com/questions/50132757/how-to-correctly-draw-simple-non-client-area-4-px-red-border
            // https://github.com/Reetus/RazorRE/blob/42f441093bd85443b39fcff5d2a02069b524b114/Crypt/Crypt.cpp
            // http://www.rohitab.com/discuss/topic/41238-global-hooks-to-intercept-windows-messages/
            // https://stackoverflow.com/questions/1146365/handling-wm-ncpaint-breaks-dwm-glass-rendering-on-vista-aero
            let data = lParam as *mut CWPSTRUCT;
            let ref data2 : CWPSTRUCT = *data;
            if data2.message == WM_NCCALCSIZE {
                let ncParamsp = data2.lParam as LPNCCALCSIZE_PARAMS;
                let ref mut ncParams : NCCALCSIZE_PARAMS = *ncParamsp;
                ncParams.rgrc[0].top += 4;
                ncParams.rgrc[0].left += 4;
                ncParams.rgrc[0].bottom -= 4;
                ncParams.rgrc[0].right -= 4;
                return 0;
            }
            else if data2.message == WM_NCACTIVATE {
                let policy = DWMNCRP_DISABLED;
				DwmSetWindowAttribute(data2.hwnd, DWMWA_NCRENDERING_POLICY, policy as LPCVOID, std::mem::size_of::<DWMNCRENDERINGPOLICY>() as DWORD);
                RedrawWindow(data2.hwnd, ptr::null_mut(), ptr::null_mut(), RDW_UPDATENOW);
                return 0;
            }
            else if data2.message == WM_NCPAINT || data2.message == WM_ERASEBKGND {
                let hwnd = data2.hwnd;

                let mut rect : RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut rect as *mut RECT);

                let mut region : HRGN = std::mem::zeroed();
                if data2.wParam as i32 == NULLREGION {
                    region = CreateRectRgn(rect.left, rect.top, rect.right, rect.bottom);
                } else {
                    let copy : HRGN = CreateRectRgn(0, 0, 0, 0);

                    let region2p = data2.lParam as *mut HRGN;
                    let region2 : HRGN = *region2p;
                    if CombineRgn(copy, region2, ptr::null_mut(), RGN_COPY) == 0 {
                        DeleteObject(copy as HGDIOBJ);
                    } else {
                        region = copy;
                    }
                }
                
                let dc = GetDCEx(hwnd, region, DCX_WINDOW | DCX_CACHE | DCX_INTERSECTRGN | DCX_LOCKWINDOWUPDATE);
                if dc == ptr::null_mut() && region != ptr::null_mut() {
                    DeleteObject(region as HGDIOBJ);
                }

                let pen : HPEN = CreatePen(PS_INSIDEFRAME as i32, 4, RGB(255, 0, 0));
                let old : HGDIOBJ = SelectObject(dc, pen as HGDIOBJ);

                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;
                Rectangle(dc, 0, 0, width, height);
                SelectObject(dc, old);
                ReleaseDC(hwnd, dc);
                DeleteObject(pen as HGDIOBJ);
                return 0;
/*
                let hdc = GetWindowDC(hwnd);
                let br = CreateSolidBrush(RGB(255,0,0));
                FillRect(hdc, &mut window, br);
                DeleteObject(br as HGDIOBJ);
                ReleaseDC(hwnd, hdc);
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("C:/Projects/winoleum/output.txt")
                    .unwrap();
                writeln!(file, "is WM_NCPAINT").unwrap();
                */

/*
				let policy = DWMNCRP_ENABLED;
                
				DwmSetWindowAttribute(hwnd, DWMWA_NCRENDERING_POLICY, policy as LPCVOID, std::mem::size_of::<DWMNCRENDERINGPOLICY>() as DWORD);
				let hdc = GetWindowDC(hwnd);
				let hdcMem = CreateCompatibleDC(hdc);

                let mut rect : RECT = std::mem::zeroed();
                let mut rect2 : RECT = std::mem::zeroed();
				GetWindowRect(hwnd, &mut rect2);
				//let r = GetTitlebarRect(hwnd);
				let hBitmap = CreateCompatibleBitmap(hdc, rect2.right, rect2.bottom);
				SelectObject(hdcMem, hBitmap as HGDIOBJ);

				rect.top = 0;
                rect.left = 0;
				rect.right = rect2.right - rect2.left;
				rect.bottom = rect2.bottom - 0;

                let br = CreateSolidBrush(RGB(255,0,0));
				FillRect(hdc, &rect, br);
				//DrawTitlebar(hdcMem, hwnd, rect, dataBuffer->titleBar);
				//BitBlt(hdc, r.left, r.top, (r.right - r.left), (r.bottom - r.top), hdcMem, 0, 0, SRCCOPY);
				//DeleteDC(hdc);
                ReleaseDC(hwnd, hdc);
                return 0;
                */  
            }
        }

        return CallNextHookEx(ptr::null_mut(), code, wParam, lParam);
    }
}