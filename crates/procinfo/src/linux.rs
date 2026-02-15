use super::LocalProcessInfo;
use std::path::PathBuf;

impl LocalProcessInfo {
    /// Linux implementation is currently not vendored in this repo.
    /// Return None to keep builds working when cross-compiling.
    pub fn with_root_pid(_pid: u32) -> Option<Self> {
        None
    }

    pub fn current_working_dir(_pid: u32) -> Option<PathBuf> {
        None
    }

    pub fn executable_path(_pid: u32) -> Option<PathBuf> {
        None
    }
}
