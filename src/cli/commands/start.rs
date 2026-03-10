use std::thread;

use anyhow::{anyhow, Result};

use crate::{
    core::{InputProxy, KeyAdapter},
    fs::{config::read_config, device::find_device_by_name},
};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

pub fn start(config_path: Option<String>) -> Result<()> {
    let parsed = read_config(config_path)?;

    let handles = parsed.keyboards.into_iter().map(|keyboard| {
        let defaults = parsed.defaults.clone();

        thread::spawn(move || -> Result<()> {
            let mut device = find_device_by_name(&keyboard.name)?
                .ok_or(anyhow!("Device not found: {}", keyboard.name))?;

            let mut proxy = InputProxy::try_from_device(&device)?;
            let mut adapter = KeyAdapter::new(keyboard, defaults, &mut proxy);

            adapter.hook(&mut device)
        })
    });

    simple_logger::init()?;

    for handle in handles {
        handle.join().unwrap()?
    }

    Ok(())
}