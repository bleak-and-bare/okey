use std::{
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::Result;

const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
const DETACHED_PROCESS: u32 = 0x00000008;

pub fn start(_: Option<String>) -> Result<()> {
    println!("I am about to run an infinite loop");
    loop {
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn start_daemon(config_path: Option<String>) -> Result<()> {
    // using `std::process::Command` because process will be started as session 1 or 2
    // Windows service are started as session 0, preventing access to event and interactivity in general

    let args = if let Some(path) = config_path {
        vec!["start".to_string(), "--config".to_string(), path]
    } else {
        vec!["start".to_string()]
    };

    let pid = Command::new(std::env::current_exe()?)
        .args(args)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .id();

    println!("okey daemon started in the background... (PID: {pid})");

    Ok(())
}
