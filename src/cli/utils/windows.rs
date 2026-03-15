use std::os::windows::ffi::OsStrExt;
use std::{ffi::OsStr, time::Duration};

use anyhow::Result;

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::HANDLE,
        Security::*,
        System::{
            RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken},
            Threading::*,
        },
    },
};

use windows_service::service::{Service, ServiceState};

pub fn wait_for_stop_for(service: &Service, timeout: Duration) -> Result<bool> {
    if let Ok(status) = service.query_status() {
        if status.current_state != ServiceState::Stopped {
            if service.stop().is_ok() {
                let start_wait = std::time::Instant::now();

                loop {
                    let status = service.query_status()?;
                    if status.current_state == ServiceState::Stopped {
                        break;
                    }

                    if start_wait.elapsed() > timeout {
                        println!("Waiting too long for service to stop.");
                        return Ok(false);
                    }

                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }

    Ok(true)
}

pub fn spawn_in_user_session() -> Result<HANDLE> {
    unsafe {
        let session_id = WTSGetActiveConsoleSessionId();

        let mut user_token = HANDLE::default();
        WTSQueryUserToken(session_id, &mut user_token)?;

        let mut primary_token = HANDLE::default();

        DuplicateTokenEx(
            user_token,
            TOKEN_ALL_ACCESS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut primary_token,
        )?;

        let cmd = format!("{} start", std::env::current_exe()?.display());
        let mut cmd: Vec<u16> = OsStr::new(&cmd)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut startup: STARTUPINFOW = std::mem::zeroed();
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

        CreateProcessAsUserW(
            Some(primary_token),
            None,
            Some(PWSTR(cmd.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_UNICODE_ENVIRONMENT,
            None,
            None,
            &startup,
            &mut process_info,
        )?;

        Ok(process_info.hProcess)
    }
}
