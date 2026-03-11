use std::{
    ffi::OsString,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Result;
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceConfig, ServiceErrorControl, ServiceInfo, ServiceStartType,
        ServiceState, ServiceStatus, ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use super::SERVICE_NAME;

pub use super::win_execute::*;

pub fn install() -> Result<()> {
    let request_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, request_access)?;
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("Okey Service"),
        service_type: ServiceType::OWN_PROCESS,
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe()?,
        launch_arguments: vec![OsString::from("service"), OsString::from("execute")],
        account_name: None,
        account_password: None,
        dependencies: vec![],
        start_type: ServiceStartType::OnDemand, // service enabled but still has to be started in `msc`
    };

    let service_access = ServiceAccess::START
        | ServiceAccess::STOP
        | ServiceAccess::DELETE
        | ServiceAccess::QUERY_CONFIG
        | ServiceAccess::QUERY_STATUS;
    let service = service_manager.create_service(&service_info, service_access)?;

    service.set_description("An advanced ready to use key remapper written in Rust.")?;
    println!("The windows service has been installed, run 'okey service start' to start it or start it within `services.msc`");

    Ok(())
}

fn wait_for_stop_for(service: &Service, timeout: Duration) -> Result<bool> {
    if let Ok(status) = service.query_status() {
        if status.current_state != ServiceState::Stopped {
            if service.stop().is_ok() {
                let start_wait = Instant::now();

                loop {
                    let status = service.query_status()?;
                    if status.current_state == ServiceState::Stopped {
                        break;
                    }

                    if start_wait.elapsed() > timeout {
                        println!("Waiting too long for service to stop.");
                        return Ok(false);
                    }

                    sleep(Duration::from_millis(500));
                }
            }
        }
    }

    Ok(true)
}

pub fn uninstall() -> Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::all())?;
    let request_access = ServiceAccess::DELETE | ServiceAccess::STOP | ServiceAccess::QUERY_STATUS;

    let service = manager.open_service(OsString::from(SERVICE_NAME), request_access)?;

    wait_for_stop_for(&service, Duration::from_secs(30))?;
    service.delete()?;
    println!("The windows service has been removed");

    Ok(())
}

pub fn start() -> Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::all())?;

    let service = manager.open_service(OsString::from(SERVICE_NAME), ServiceAccess::START)?;
    service.start::<OsString>(&[])?;

    Ok(())
}

pub fn stop() -> Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::all())?;

    let service = manager.open_service(OsString::from(SERVICE_NAME), ServiceAccess::STOP)?;
    service.stop()?;

    Ok(())
}

pub fn restart() -> Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service_access = ServiceAccess::START | ServiceAccess::STOP | ServiceAccess::QUERY_STATUS;
    let service = manager.open_service(SERVICE_NAME, service_access)?;

    if wait_for_stop_for(&service, Duration::from_secs(30))? {
        service.start::<OsString>(&[])?;
    }

    Ok(())
}

fn service_status_description(status: &ServiceStatus) -> &'static str {
    match status.current_state {
        ServiceState::Stopped => "❌ stopped",
        ServiceState::StartPending => "starting",
        ServiceState::StopPending => "stopping",
        ServiceState::Running => "🟢 running",
        ServiceState::ContinuePending => "continuing",
        ServiceState::PausePending => "pausing",
        ServiceState::Paused => "paused",
    }
}

fn service_start_type(config: &ServiceConfig) -> &'static str {
    match config.start_type {
        ServiceStartType::AutoStart => "automatic",
        ServiceStartType::BootStart => "boot",
        ServiceStartType::Disabled => "disabled",
        ServiceStartType::OnDemand => "on demand",
        ServiceStartType::SystemStart => "system",
    }
}

pub fn status() -> Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(
        SERVICE_NAME,
        ServiceAccess::QUERY_STATUS | ServiceAccess::QUERY_CONFIG,
    )?;

    let status = service.query_status()?;
    let config = service.query_config()?;

    // TODO : print something prettier

    println!("Service : {}", SERVICE_NAME);
    println!("Display name : {}", config.display_name.to_string_lossy());
    println!("State : {}", service_status_description(&status));
    println!("Start type : {}", service_start_type(&config));

    Ok(())
}
