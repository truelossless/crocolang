use std::process::{Command, Stdio};

use crate::ms_craziness_bindings::find_msvc;

/// a linker to transform object files into an executable
// this leverages native linkers
pub struct Linker {
    linker: String,
    // if the linker is link.exe, give the location of the runtime library
    has_msvcrt: Option<String>,
}

impl Linker {
    pub fn new() -> Self {
        Linker {
            linker: String::new(),
            has_msvcrt: None,
        }
    }

    pub fn find_linker(&mut self) -> Result<String, String> {
        // common linkers that can be found on the machine
        let linkers = vec!["lld", "cc", "clang", "gcc", "ld"];

        // we are going to locate the linker
        for linker in linkers {
            let mut command;

            if cfg!(windows) {
                // on windows string escaping with cmd is bugged:
                // we can bypass the cmd spawn by getting the full linker executable path with the where command.
                let linker_search = format!("where {}", linker);
                command = Command::new("cmd");
                command.args(&["/C", &linker_search]);
            } else {
                // on linux this is not needed
                let linker_search = format!("{} --version", linker);
                command = Command::new("sh");
                command.args(&["-c", &linker_search]);
            };

            command.stderr(Stdio::null());
            let output = command.stdout(Stdio::piped()).output().unwrap();

            if let Ok(status) = command.status() {
                if status.success() {
                    // if we are on windows we can retreive the full path in the stdout of the where command
                    if cfg!(windows) {
                        self.linker = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .to_owned();
                    } else {
                        self.linker = linker.to_owned();
                    }

                    return Ok(format!("Linker found: {}", self.linker));
                }
            }
        }

        // if we are on windows we can try to locate MSVC tools
        if cfg!(windows) {
            let msvc_result = find_msvc();

            if msvc_result.windows_sdk_version != 0 {
                self.linker = format!("{}\\link.exe", msvc_result.vs_exe_path);
                self.has_msvcrt = Some(msvc_result.vs_library_path);
                return Ok("Linker found: link.exe".to_owned());
            }
        }

        Err("No suitable linker found".to_owned())
    }

    pub fn link(&self, object_file: &str, output_file: &str) -> Result<String, String> {
        let mut command;

        if cfg!(windows) {
            command = Command::new(&self.linker);
            command.arg(&object_file);

            // if we're using msvc
            if let Some(msvcrt_path) = &self.has_msvcrt {
                command.arg(format!("{}\\msvcrt.lib", msvcrt_path));
                command.arg(format!("/OUT:{}", output_file));
                command.arg("/ENTRY:main");
                command.arg("/NOLOGO");
            } else {
                command.args(&["-o", output_file]);
            }
        } else {
            let link_command = format!("{} \"{}\" \"{}\"", self.linker, object_file, output_file);
            command = Command::new("sh");
            command.args(&["-c", &link_command]);
        }

        if let Ok(status) = command.status() {
            if status.success() {
                return Ok(format!("Executable built under {}", object_file));
            }
        }

        Err("linking failed !".to_owned())
    }
}
