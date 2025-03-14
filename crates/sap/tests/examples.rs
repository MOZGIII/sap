//! Test the examples.

use std::path::Path;

use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _};

const EXECUTABLE_PATH: &str = env!("CARGO_BIN_EXE_sap");

async fn read_envs(path: impl AsRef<Path>) -> Vec<(String, String)> {
    let result = tokio::fs::read_to_string(path).await;

    let data = match result {
        Ok(v) => v,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Vec::new();
        }
        Err(err) => panic!("{:?}", err),
    };

    let parsed: std::collections::HashMap<String, String> = serde_yaml::from_str(&data).unwrap();
    parsed.into_iter().collect()
}

async fn prepare_sap_command(
    example_dir: impl AsRef<Path>,
    mode: &'static str,
) -> tokio::process::Command {
    let example_dir_path = example_dir.as_ref();

    let envs = read_envs(example_dir_path.join("env.yaml")).await;

    let mut command = tokio::process::Command::new(EXECUTABLE_PATH);

    command
        .current_dir(example_dir_path)
        .env("MODE", mode)
        .env("ROOT_DIR", example_dir_path.join("root"))
        .env("RUST_LOG", "debug")
        .envs(envs);

    command
}

#[tokio::test]
async fn check_examples() {
    let examples = std::fs::read_dir("../../examples").unwrap();

    for example in examples {
        let example = example.unwrap();

        let example_path = example.path();
        let mut command = prepare_sap_command(&example_path, "check").await;

        command.stderr(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.kill_on_drop(true);

        println!("Example: {example_path:?}");

        let child = command.spawn().unwrap();

        let std::process::Output {
            status,
            stdout,
            stderr,
        } = child.wait_with_output().await.unwrap();

        println!("Status: {status}");
        println!("Stderr:\n{}", String::from_utf8_lossy(&stderr));
        println!("Stdout:\n{}", String::from_utf8_lossy(&stdout));

        assert!(status.success());
    }
}

#[tokio::test]
async fn test_examples_with_hurl() {
    let examples = std::fs::read_dir("../../examples").unwrap();

    let mut readline_buf = String::new();

    for example in examples {
        let example = example.unwrap();

        let example_path = example.path();
        let mut command = prepare_sap_command(&example_path, "run").await;

        command.stderr(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.kill_on_drop(true);

        println!("Example: {example_path:?}");

        let mut child = command.spawn().unwrap();

        let stdout_reader = child.stdout.take().unwrap();
        let mut stdout = Vec::new();
        let mut stdout_reader = tokio::io::BufReader::new(stdout_reader);

        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                readline_buf.clear();
                let size = stdout_reader.read_line(&mut readline_buf).await.unwrap();
                readline_buf.truncate(size);
                stdout.extend_from_slice(readline_buf.as_bytes());

                print!("{readline_buf}");

                if readline_buf.contains("Started Tcp listening on") {
                    break;
                }
            }
        })
        .await
        .unwrap();

        let hurl = tokio::process::Command::new("hurl")
            .arg("--test")
            .arg("--verbose")
            .arg(example_path.join("test.hurl"))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();

        let std::process::Output {
            status: hurl_status,
            stdout: hurl_stdout,
            stderr: hurl_stderr,
        } = hurl.wait_with_output().await.unwrap();

        child.start_kill().unwrap();

        let std::process::Output {
            status,
            stdout: _,
            stderr,
        } = child.wait_with_output().await.unwrap();
        stdout_reader.read_to_end(&mut stdout).await.unwrap();

        println!("Status: {status}");
        println!("Stderr:\n{}", String::from_utf8_lossy(&stderr));
        println!("Stdout:\n{}", String::from_utf8_lossy(&stdout));

        println!("HURL Status: {hurl_status}");
        println!("HURL Stderr:\n{}", String::from_utf8_lossy(&hurl_stderr));
        println!("HURL Stdout:\n{}", String::from_utf8_lossy(&hurl_stdout));

        assert!(hurl_status.success());
    }
}
