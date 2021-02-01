mod arrays;
mod conditions;
mod functions;
mod loops;
mod primitives;
mod references;
mod structs;

use std::process::Command;

pub enum Backend {
    Crocoi,
    Crocol,
}

pub const CROCOI: &[Backend] = &[];
pub const ALL_BACKENDS: &[Backend] = &[Backend::Crocoi, Backend::Crocol];

pub fn test_file_ok(path: &str, backends: &[Backend]) {
    test_file(path, backends, true);
}

pub fn test_file_err(path: &str, backends: &[Backend]) {
    test_file(path, backends, false);
}

fn test_file(path: &str, backends: &[Backend], should_succeed: bool) {
    for backend in backends {
        #[allow(clippy::clippy::single_match)]
        match backend {
            #[cfg(feature = "crocoi")]
            Backend::Crocoi => {
                let mut crocoi = croco::Crocoi::new();
                let res = crocoi.exec_file(path);

                if should_succeed {
                    res.unwrap();
                } else {
                    res.unwrap_err();
                }
            }

            #[cfg(feature = "crocol")]
            Backend::Crocol => {
                let tmp_exe_path = format!("{}tmp", path);
                let mut crocol = croco::Crocol::new();
                crocol.set_output(tmp_exe_path.clone());
                let comp_res = crocol.exec_file(path);

                if should_succeed {
                    comp_res.as_ref().unwrap();
                }

                if comp_res.is_ok() {
                    let runtime_res = Command::new(&tmp_exe_path).status().unwrap();
                    std::fs::remove_file(tmp_exe_path).unwrap();

                    if should_succeed {
                        // for now we don't return any exit code on success.
                        // the exit code is therefore random, and there's a small chance
                        // that we stumble into the error code (1) while doing so ...
                        assert!(runtime_res.code() != Some(1));
                    } else {
                        assert!(runtime_res.code() == Some(1));
                    }
                }
            }

            // this is hit if we try to execute one backend-specific test when
            // the feature flag for that backend isn't enabled. Just pass the test.
            #[allow(unreachable_patterns)]
            _ => (),
        }
    }
}
