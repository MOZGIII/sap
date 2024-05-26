use std::path::Path;

const EXECUTABLE_PATH: &str = env!("CARGO_BIN_EXE_sap");

fn prepare_command(example_dir: impl AsRef<Path>) -> std::process::Command {
    let mut command = std::process::Command::new(EXECUTABLE_PATH);

    command
        .env("MODE", "check")
        .env("ROOT_DIR", example_dir.as_ref())
        .env("RUST_LOG", "debug");

    command
}

#[test]
fn check_examples() {
    let examples = std::fs::read_dir("../../examples").unwrap();

    for example in examples {
        let example = example.unwrap();

        let example_path = example.path();
        let mut command = prepare_command(&example_path);

        command.stderr(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());

        println!("Example: {example_path:?}");

        let child = command.spawn().unwrap();

        let std::process::Output {
            status,
            stdout,
            stderr,
        } = child.wait_with_output().unwrap();

        println!("Status: {status}");
        println!("Stderr:\n{}", String::from_utf8_lossy(&stderr));
        println!("Stdout:\n{}", String::from_utf8_lossy(&stdout));

        assert!(status.success());
    }
}
