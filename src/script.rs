use std::ffi::OsStr;
use std::io::Write;

use tempfile::TempPath;

use crate::{Result, Shell, ShellError};

pub enum Script {
    Inline(String),
    File(TempPath),
}

impl Script {
    pub async fn build(shell: Shell, cmd: String, init: Option<String>) -> Result<Self> {
        let init_script = match init.as_ref().map(|s| s.trim()) {
            Some(init) if !init.is_empty() => init_line(init, shell),
            _ => match shell {
                Shell::Bash => init_line("source ~/.bashrc", shell),
                Shell::Zsh => init_line("source ~/.zshrc", shell),
                Shell::Cmd => "@echo off".to_string(),
                _ => "".to_string(),
            },
        };

        let raw = fix_newlines(shell, &format!("{init_script}\n{cmd}"));

        let cmd = match shell {
            Shell::Cmd => {
                let file = write_file(raw).await?;
                Self::File(file)
            }
            _ => Self::Inline(raw),
        };
        Ok(cmd)
    }
}

impl AsRef<OsStr> for &Script {
    fn as_ref(&self) -> &OsStr {
        match self {
            Script::Inline(v) => v.as_ref(),
            Script::File(v) => v.as_os_str(),
        }
    }
}

fn init_line(script: &str, shell: Shell) -> String {
    match shell {
        Shell::Cmd => format!("{script} 2> nul"),
        Shell::Powershell => format!("{script} 2>$null"),
        Shell::Bash | Shell::Zsh | Shell::Sh | Shell::Wsl => format!("{script} > /dev/null 2>&1"),
    }
}

fn fix_newlines(shell: Shell, script: &str) -> String {
    let separator = match shell {
        Shell::Cmd | Shell::Powershell => "\r\n",
        _ => "\n",
    };
    script.lines().collect::<Vec<&str>>().join(separator)
}

async fn write_file(raw: String) -> Result<TempPath> {
    tokio::task::spawn_blocking(move || {
        let mut file = tempfile::Builder::new().suffix(".bat").tempfile()?;
        file.write_all(raw.as_bytes())?;
        let a = file.into_temp_path();
        Ok(a)
    })
    .await
    .map_err(ShellError::FailedJoin)?
    .map_err(ShellError::FailedPrepare)
}
