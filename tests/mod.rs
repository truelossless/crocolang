mod arrays;
mod conditions;
mod functions;
mod primitives;
mod references;
mod structs;

use std::process::Command;
use croco::{Crocoi, Crocol};

pub enum Backend {
    Crocoi,
    Crocol,
}

pub static CROCOI: &[Backend] = &[Backend::Crocoi];
pub static ALL_BACKENDS: &[Backend] = &[Backend::Crocoi, Backend::Crocol];

pub fn test_file_ok(path: &str, backends: &[Backend]) {
    test_file(path, backends, true);
}

pub fn test_file_err(path: &str, backends: &[Backend]) {
    test_file(path, backends, false);
}

fn test_file(path: &str, backends: &[Backend], should_succeed: bool) {
    for backend in backends {
        match backend {
            Backend::Crocoi => {
                let mut crocoi = Crocoi::new();
                let res = crocoi.exec_file(path);

                if should_succeed {
                    res.unwrap();
                } else {
                    res.unwrap_err();
                }
            }

            Backend::Crocol => {
                let tmp_exe_path = format!("{}tmp", path);
                let mut crocol = Crocol::new();
                crocol.set_output(tmp_exe_path.clone());
                let comp_res = crocol.exec_file(path);

                if should_succeed {
                    comp_res.as_ref().unwrap();
                }

                if comp_res.is_ok() {
                    let runtime_res = Command::new(&tmp_exe_path).status().unwrap();
                    std::fs::remove_file(tmp_exe_path).unwrap();

                    if should_succeed {
                        assert!(runtime_res.success());
                    } else {
                        assert!(!runtime_res.success());
                    }
                }
            }
        }
    }
}