use std::ffi::OsStr;

use tokio::process::Command;

pub enum Argument<'a> {
    Normal(&'a str),
    Path(&'a OsStr),
    Raw(&'a str),
}

pub trait CommandArgument {
    fn argument<'a>(&mut self, arg: &Argument<'a>) -> &mut Self;
}

impl CommandArgument for Command {
    fn argument<'a>(&mut self, arg: &Argument<'a>) -> &mut Self {
        match arg {
            Argument::Normal(value) => self.arg(value),
            Argument::Path(value) => self.arg(value),
            Argument::Raw(value) => {
                #[cfg(windows)]
                {
                    self.raw_arg(value)
                }
                #[cfg(unix)]
                {
                    self.arg(value)
                }
            }
        }
    }
}
