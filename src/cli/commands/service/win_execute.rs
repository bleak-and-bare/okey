use std::{ffi::OsString, time::Duration};

use anyhow::Result;
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

fn run_service() -> Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => ServiceControlHandlerResult::NoError,
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
    let next_status = ServiceStatus {
        checkpoint: 0,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::PAUSE_CONTINUE,
        current_state: ServiceState::Running,
        exit_code: ServiceExitCode::Win32(0),
        process_id: None,
        service_type: ServiceType::OWN_PROCESS,
        wait_hint: Duration::default(),
    };

    status_handle.set_service_status(next_status)?;

    // do shit here

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
