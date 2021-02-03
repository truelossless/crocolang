use std::{env, path::Path, process::Command};

/// Checks if a file is present is path
fn file_in_path(file_name: &str) -> Option<String> {
    let path = env::var_os("PATH").unwrap();

    for folder in env::split_paths(&path) {
        let file = Path::new(&folder).join(file_name);
        if file.exists() {
            return Some(file.to_string_lossy().into());
        }
    }

    // we found nothing
    None
}

/// Links an object file on Windows
#[cfg(windows)]
pub fn link(object_file: &str, output_file: &str) -> Result<String, String> {
    let mut command;
    let linker_path;

    // when possible, always use clang or gcc as the linker driver.
    let gnulike_cc = file_in_path("clang.exe").or_else(|| file_in_path("gcc.exe"));

    if let Some(cc) = gnulike_cc {
        linker_path = cc;
        command = Command::new(&linker_path);
        command.args(&[object_file, "-o", output_file]);

    // else try to detect ourselves the msvc toolchain
    } else {
        // SAFETY: this leaks a string but it is freed by the OS when the linker finishes.
        let msvc_result = unsafe { crate::ms_craziness_bindings::find_msvc() }
            .ok_or_else(|| "no suitable linker found".to_owned())?;

        linker_path = format!("{}\\link.exe", &msvc_result.vs_exe_path);
        let msvc_libs = [
            format!("{}/msvcrt.lib", msvc_result.vs_library_path),
            format!("{}/vcruntime.lib", msvc_result.vs_library_path),
            format!("{}/uuid.lib", msvc_result.windows_sdk_um_library_path),
            format!("{}/kernel32.lib", msvc_result.windows_sdk_um_library_path),
            format!("{}/ucrt.lib", msvc_result.windows_sdk_ucrt_library_path),
        ];

        command = Command::new(&linker_path);
        command.arg(&object_file);
        command.args(&msvc_libs);
        command.arg(format!("/OUT:{}", output_file));
        command.arg("/ENTRY:main");
        command.arg("/NOLOGO");
    }

    if let Ok(status) = command.status() {
        if status.success() {
            return Ok(format!("Executable built with {}", linker_path));
        }
    }

    Err(format!("linking failed with {}!", linker_path))
}

/// Links an object file on Unix
#[cfg(not(windows))]
pub fn link(object_file: &str, output_file: &str) -> Result<String, String> {
    // find a compiler to act as the linker
    // as opposed to Windows, we are compelled to rely on this,
    // because linking by hand with `ld` is almost impossible.
    // C compilers handle all the extra logic to detect libc paths
    // and extra arguments. For reference, see the linux-specific code
    // used only on Linux targets in Clang !
    // https://clang.llvm.org/doxygen/Linux_8cpp_source.html
    let gnulike_cc = file_in_path("clang.exe")
        .or_else(|| file_in_path("gcc.exe"))
        .ok_or("no suitable linker found")?;

    let mut command = Command::new(&gnulike_cc);
    command.args(&[object_file, "-o", output_file]);

    if let Ok(status) = command.status() {
        if status.success() {
            return Ok(format!("Executable built with {}", gnulike_cc));
        }
    }

    Err(format!("linking failed with {}!", gnulike_cc))
}
