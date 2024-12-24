use strum::{Display, EnumString};

use crate::Argument;

#[derive(Debug, EnumString, Display, Copy, Clone)]
pub enum Shell {
    #[strum(serialize = "zsh")]
    Zsh,
    #[strum(serialize = "bash")]
    Bash,
    #[strum(serialize = "sh")]
    Sh,
    #[strum(serialize = "cmd")]
    Cmd,
    #[strum(serialize = "powershell")]
    Powershell,
    #[strum(serialize = "wsl")]
    Wsl,
}

impl Shell {
    pub fn command_args(&self) -> &[Argument<'static>] {
        match self {
            Self::Cmd => &[Argument::Normal("/C")],
            Self::Powershell => &[Argument::Normal("-Command")],
            Self::Wsl => &[Argument::Normal("bash"), Argument::Normal("-c")],
            _ => &[Argument::Normal("-c")],
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            Self::Cmd
        } else {
            Self::Bash
        }
    }
}
