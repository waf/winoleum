extern crate winapi;

use std::ptr;
use std::fs;
use winapi::um::winuser::{ CallNextHookEx };

#[no_mangle]
pub extern "system" fn hook_callback(code: i32, wParam: usize, lParam: isize) -> isize {
    unsafe {
        let data = format!("Received code: {}", code);
        fs::write("C:/Projects/win10wm/output.txt", data).expect("Unable to write file");

        return CallNextHookEx(ptr::null_mut(), code, wParam, lParam);
    }
}