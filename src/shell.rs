use strum::{Display, EnumString};

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
    pub fn command_arg(&self) -> Option<&'static str> {
        match self {
            Self::Cmd => Some("/C"),
            Self::Powershell => Some("-Command"),
            Self::Wsl => None,
            _ => Some("-c"),
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
