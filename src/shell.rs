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
}

impl Shell {
    pub fn command_arg<'a>(&self) -> &'a str {
        match self {
            Self::Cmd => "/C",
            Self::Powershell => "-Command",
            _ => "-c",
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
