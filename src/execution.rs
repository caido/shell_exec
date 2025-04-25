use std::ffi::OsStr;
use std::iter;
use std::process::Stdio;
use std::time::Duration;

use bstr::ByteSlice;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;
use typed_builder::TypedBuilder;

use crate::{CommandArgument, EnvCollector, Result, Script, Shell, ShellError};

#[derive(TypedBuilder)]
pub struct Execution {
    shell: Shell,
    timeout: Duration,
    cmd: String,
    #[builder(default)]
    init: Option<String>,
}

impl Execution {
    pub async fn execute(self, data: &[u8]) -> Result<Vec<u8>> {
        self.execute_with_envs(data, iter::empty::<(&str, &str)>())
            .await
    }

    pub async fn execute_with_envs<I, K, V>(self, data: &[u8], envs: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        // Prepare script
        let script = Script::build(self.shell, self.cmd, self.init).await?;

        // Spawn
        // NOTE: If kill_on_drop is proven not sufficiently reliable, we might want to explicitly kill the process
        // before exiting the function. This approach is slower since it awaits the process termination.
        let mut builder = Command::new(self.shell.to_string());
        for arg in self.shell.command_args() {
            builder.argument(arg);
        }
        let mut env_collector = EnvCollector::new(self.shell);
        for (key, val) in envs {
            env_collector.acc(key.as_ref(), val.as_ref());
            builder.env(key, val);
        }
        if let Some((key, val)) = env_collector.collect() {
            builder.env(key, val);
        }
        let mut cmd_handle = builder
            .argument(&script.argument())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(ShellError::FailedSpawn)?;

        // Write STDIN
        let mut stdin = cmd_handle
            .stdin
            .take()
            .expect("Stdin missing for cmd handle");

        let write_stdin = async move {
            if let Err(err) = stdin.write_all(data).await {
                log::error!(target: "Shell", "Failed to write data to shell: {:?}", err);
            }
            drop(stdin);
            Ok(())
        };

        // Get output
        let get_output = async move {
            let timeout_output = timeout(self.timeout, cmd_handle.wait_with_output())
                .await
                .map_err(|_e| ShellError::Timeout)?;
            let output = timeout_output.map_err(ShellError::FailedOutput)?;

            if output.status.success() {
                let stripped_cmd_stdout = output.stdout.trim_end().to_vec();
                Ok(stripped_cmd_stdout)
            } else {
                Err(ShellError::Failure(
                    String::from_utf8_lossy(&output.stderr).into_owned(),
                ))
            }
        };

        // Wait for result
        // NOTE: Here, write_stdin will never complete the select since it always returns Ok.
        // This allows us to avoid spawning a task for it, but we don't care what is does.
        tokio::select! {
          Err(err) = write_stdin => Err(err),
          result = get_output => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(unix)]
    async fn should_execute_sh() {
        let execution = Execution::builder()
            .shell(Shell::Sh)
            .cmd(r#"jq -r .hello"#.to_string())
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"{\"hello\":\"world\"}").await.unwrap();

        assert_eq!(b"world"[..], data);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn should_execute_bash() {
        let execution = Execution::builder()
            .shell(Shell::Bash)
            .cmd(
                r#"
                INPUT=`cat -`;
                echo "hello $INPUT"
                "#
                .to_string(),
            )
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"world").await.unwrap();

        assert_eq!(b"hello world"[..], data);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn should_execute_with_envs() {
        let execution = Execution::builder()
            .shell(Shell::Sh)
            .cmd(r#"echo $TEST | jq -r .hello"#.to_string())
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution
            .execute_with_envs(b"", [("TEST", "{\"hello\":\"world\"}")])
            .await
            .unwrap();

        assert_eq!(b"world"[..], data);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn should_execute_init_script() {
        let execution = Execution::builder()
            .shell(Shell::Sh)
            .cmd(r#"echo "$TEST""#.to_string())
            .init(Some(r#"export TEST="HELLO WORLD!""#.to_string()))
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"").await.unwrap();

        assert_eq!(b"HELLO WORLD!"[..], data);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn should_return_err() {
        let execution = Execution::builder()
            .shell(Shell::Sh)
            .cmd(r#"nonexistantcommand"#.to_string())
            .timeout(Duration::from_millis(10000))
            .build();

        let err = execution.execute(b"").await.unwrap_err();

        assert!(matches!(err, ShellError::Failure(_)));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn should_timeout() {
        let execution = Execution::builder()
            .shell(Shell::Sh)
            .cmd(r#"sleep 5"#.to_string())
            .timeout(Duration::from_millis(200))
            .build();

        let err = execution.execute(b"").await.unwrap_err();

        assert!(matches!(err, ShellError::Timeout));
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn should_execute_cmd() {
        let execution = Execution::builder()
            .shell(Shell::Cmd)
            .cmd(
                r#"
                set /p input=""
                echo hello %input%
                "#
                .to_string(),
            )
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"world").await.unwrap();

        assert_eq!(b"hello world"[..], data);
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn should_execute_powershell() {
        let execution = Execution::builder()
            .shell(Shell::Powershell)
            .cmd(
                r#"
                $InputValue = $input | ForEach-Object ToUpper
                Write-Host "hello`n & $InputValue"
                "#
                .to_string(),
            )
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"world").await.unwrap();

        assert_eq!(b"hello\n & WORLD"[..], data);
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn should_execute_wsl() {
        let execution = Execution::builder()
            .shell(Shell::Wsl)
            .cmd(
                r#"
                INPUT=$(cat);
                echo "hello $INPUT"
                "#
                .to_string(),
            )
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution.execute(b"world").await.unwrap();

        assert_eq!(b"hello world"[..], data);
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn should_execute_wsl_env() {
        let execution = Execution::builder()
            .shell(Shell::Wsl)
            .cmd(
                r#"
                echo "hello $INPUT"
                "#
                .to_string(),
            )
            .timeout(Duration::from_millis(10000))
            .build();

        let data = execution
            .execute_with_envs(b"", [("INPUT", "world")])
            .await
            .unwrap();

        assert_eq!(b"hello world"[..], data);
    }
}
