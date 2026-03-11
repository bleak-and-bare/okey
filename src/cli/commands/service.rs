#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
mod win_execute;

#[cfg(target_os = "windows")]
pub use windows::*;

pub const SERVICE_NAME: &'static str = "Okey Service";