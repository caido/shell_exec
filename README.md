# Shell Exec
[<img alt="github" src="https://img.shields.io/badge/github-caido/shell_exec-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/caido/shell_exec)
[<img alt="crates.io" src="https://img.shields.io/crates/v/shell_exec?color=fc8d62&logo=rust&style=for-the-badge" height="20">](https://crates.io/crates/shell_exec)

Execute shell scripts asynchronously on multiple platforms.
The goal of the library is to provide a simple interface to execute a user provided script on any shell.
We try to avoid as much as possible writing temporary files.

```rust
use std::time::Duration;
use shell_exec::{Execution, Shell};

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
```
