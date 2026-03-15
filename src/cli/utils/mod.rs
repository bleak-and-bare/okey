#[cfg(target_os = "linux")]
pub mod device;

#[cfg(target_os = "linux")]
pub mod systemctl;

#[cfg(target_os = "windows")]
pub mod windows;