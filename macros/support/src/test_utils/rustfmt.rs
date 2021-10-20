use std::io::Write;
use std::process::{Command, Stdio};

/// Runs a text string through `rustfmt`. If anything goes wrong, returns the
/// original string. This should be used only for debugging.
pub fn run(input: &str) -> String {
    let try_block = || -> std::io::Result<String> {
        let mut cmd = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = cmd.stdin.take().unwrap();

        let input_string = input.to_string();

        let stdin_thread = std::thread::spawn(move || {
            let _ = stdin.write(input_string.as_bytes());
            // if it fails, it fails
        });

        let output = cmd.wait_with_output()?;

        // Join on the stdin thread after the child process exits.
        // Otherwise, deadlock is possible.
        stdin_thread.join().unwrap();

        if !output.status.success() {
            eprintln!("rustfmt failed");
            return Ok(input.to_string());
        }

        Ok(std::str::from_utf8(&output.stdout).unwrap().to_string())
    };

    match try_block() {
        Ok(formatted) => formatted,
        Err(e) => {
            eprintln!("warning: rustfmt failed: {:?}", e);
            input.to_string()
        }
    }
}
