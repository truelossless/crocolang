extern crate libc;

use std::ffi::OsString;
use std::os::windows::prelude::*;

use libc::c_int;
use libc::c_void;
use libc::wchar_t;

#[repr(C)]
pub struct Find_Result {
    pub windows_sdk_version: c_int,
    pub windows_sdk_root: *mut wchar_t,
    pub windows_sdk_um_library_path: *mut wchar_t,
    pub windows_sdk_ucrt_library_path: *mut wchar_t,
    pub vs_exe_path: *mut wchar_t,
    pub vs_library_path: *mut wchar_t,
}

extern "C" {
    pub fn find_visual_studio_and_windows_sdk() -> Find_Result;
    pub fn free_resources(result: *mut Find_Result) -> c_void;
}

pub struct FindResult {
    pub windows_sdk_version: i32,
    pub windows_sdk_root: String,
    pub windows_sdk_um_library_path: String,
    pub windows_sdk_ucrt_library_path: String,
    pub vs_exe_path: String,
    pub vs_library_path: String,
}


// https://stackoverflow.com/questions/48586816/converting-raw-pointer-to-16-bit-unicode-character-to-file-path-in-rust
// since this is only ran on windows machines, we can assume that wchar_t is 16bit wide.
unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);

    OsString::from_wide(slice)
}

// this is marked as unsafe as one SysFreeString() call has been commented out in the C source - it was throwing an unresolved symbol error.
// this memory leak doesn't matter that much since after the linker step the program will exit and the OS free itself all the allocated memory.
pub fn find_msvc() -> FindResult {
    let mut c_find_result = unsafe { find_visual_studio_and_windows_sdk() };

    let mut find_result = FindResult {
        windows_sdk_version: c_find_result.windows_sdk_version,
        windows_sdk_root: String::new(),
        windows_sdk_ucrt_library_path: String::new(),
        windows_sdk_um_library_path: String::new(),
        vs_exe_path: String::new(),
        vs_library_path: String::new(),
    };

    if find_result.windows_sdk_version == 0 {
        return find_result;
    }

    find_result.vs_exe_path = unsafe { u16_ptr_to_string(c_find_result.vs_exe_path) }
        .to_string_lossy()
        .into();
    find_result.windows_sdk_root = unsafe { u16_ptr_to_string(c_find_result.windows_sdk_root) }
        .to_string_lossy()
        .into();
    find_result.windows_sdk_ucrt_library_path =
        unsafe { u16_ptr_to_string(c_find_result.windows_sdk_ucrt_library_path) }
            .to_string_lossy()
            .into();
    find_result.windows_sdk_um_library_path =
        unsafe { u16_ptr_to_string(c_find_result.windows_sdk_um_library_path) }
            .to_string_lossy()
            .into();
    find_result.vs_library_path = unsafe { u16_ptr_to_string(c_find_result.vs_library_path) }
        .to_string_lossy()
        .into();

    unsafe {
        free_resources(&mut c_find_result);
    }

    find_result
}