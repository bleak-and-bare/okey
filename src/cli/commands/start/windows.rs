use std::{os::windows::process::CommandExt, process::Command};

use anyhow::Result;

const DETACHED_PROCESS: u32 = 0x00000008;

pub fn start_daemon(config_path: Option<String>) -> Result<()> {
    let args = if let Some(path) = config_path {
        vec!["start".to_string(), "--config".to_string(), path]
    } else {
        vec!["start".to_string()]
    };
    
    Command::new(std::env::current_exe()?)
        .args(args)
        .creation_flags(DETACHED_PROCESS)
        .spawn()?;

    Ok(())
}
