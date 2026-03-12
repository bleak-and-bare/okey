use std::{ffi::{OsStr, OsString}, os::windows::ffi::OsStrExt, time::Duration};

use anyhow::Result;
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::HANDLE,
        Security::{DuplicateTokenEx, SecurityImpersonation, TokenPrimary, TOKEN_ALL_ACCESS},
        System::{
            RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken},
            Threading::{
                CreateProcessAsUserW, TerminateProcess, CREATE_UNICODE_ENVIRONMENT,
                PROCESS_INFORMATION, STARTUPINFOW,
            },
        },
    },
};

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

use super::SERVICE_NAME;

define_windows_service!(ffi_service_main, service_main);

fn spawn_in_user_session() -> Result<HANDLE> {
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

fn run_service() -> Result<()> {
    // Reference : https://learn.microsoft.com/en-us/windows/win32/services/service-status-transitions

    let (tx, rx) = std::sync::mpsc::channel();

    // Aknowledged commands specified here :`crate::cli::commands::service::windows::install`
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // run process as session 1
    let process = spawn_in_user_session()?;

    let mut next_status = ServiceStatus {
        checkpoint: 0,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::PAUSE_CONTINUE,
        current_state: ServiceState::Running,
        exit_code: ServiceExitCode::Win32(0),
        process_id: None,
        service_type: ServiceType::OWN_PROCESS,
        wait_hint: Duration::default(),
    };

    // RUNNING
    status_handle.set_service_status(next_status.clone())?;

    rx.recv().unwrap();

    unsafe {
        TerminateProcess(process, 0)?;
    }

    next_status.current_state = ServiceState::Stopped;
    next_status.controls_accepted = ServiceControlAccept::empty();
    
    // STOPPED
    status_handle.set_service_status(next_status.clone())?;

    Ok(())
}

fn service_main(_: Vec<OsString>) {
    if let Err(err) = run_service() {
        eprintln!("Okey service exited with an error : {:?}", err);
    }
}

pub fn execute() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
