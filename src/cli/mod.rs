pub mod commands;

mod utils;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Start the keyboard remapping hook
    Start {
        /// Configuration file path (default: ~/.config/okey/config.yaml)
        #[arg(short, long)]
        config: Option<String>,
        /// Whether to start the process as a daemon
        #[arg(short, long, default_value_t = false)]
        daemon: bool,

        #[arg(
            long,
            hide_short_help = true,
            hide_long_help = true,
            default_value_t = false
        )]
        systemd: bool,
    },

    /// Utility commands for the systemd service
    Service {
        #[command(subcommand)]
        command: SystemdSubcommand,
    },

    /// Utility commands for debugging input devices
    Device {
        #[command(subcommand)]
        command: DeviceSubcommand,
    },
}

#[derive(Parser, Debug)]
pub enum SystemdSubcommand {
    #[cfg_attr(
        target_os = "windows",
        doc = "Shorthand for 'sc start \"Okey Service\"'"
    )]
    #[cfg_attr(
        target_family = "unix",
        doc = "Shorthand for 'systemctl --user enable okey && systemctl --user start okey'"
    )]
    Start,

    #[cfg_attr(
        target_os = "windows",
        doc = "Shorthand for 'sc stop \"Okey Service\"'"
    )]
    #[cfg_attr(
        target_family = "unix",
        doc = "Shorthand for 'systemctl --user stop okey && systemctl --user disable okey'"
    )]
    Stop,

    #[cfg_attr(
        target_os = "windows",
        doc = "Shorthand for 'sc stop \"Okey Service\" && sc start \"Okey Service\"'"
    )]
    #[cfg_attr(
        target_family = "unix",
        doc = "Shorthand for 'systemctl --user restart okey'"
    )]
    Restart,

    #[cfg_attr(
        target_os = "windows",
        doc = "Shorthand for 'sc queryex \"Okey Service\"'"
    )]
    #[cfg_attr(
        target_family = "unix",
        doc = "Shorthand for 'systemctl --user status okey'"
    )]
    Status,

    #[cfg_attr(target_os = "windows", doc = "Register to windows service manager")]
    #[cfg_attr(target_family = "unix", doc = "Create the systemd service file")]
    Install,

    #[cfg_attr(target_os = "windows", doc = "Disable and remove the service")]
    #[cfg_attr(target_family = "unix", doc = "Disable and remove the service file")]
    Uninstall,

    #[cfg(target_os = "windows")]
    #[command(hide = true)]
    /// Execute windows service main
    Execute,
}

#[derive(Parser, Debug)]
pub enum DeviceSubcommand {
    /// List all input devices that support keys
    List {
        /// Whether to only show keyboards
        #[arg(short, long, default_value_t = false)]
        keyboard: bool,
    },
}
