use std::process::Command;

fn main() {
    let mut child_proc = Command::new("cmd")
        .args(&["/C", "cls && cargo build --all --release"])
        .spawn()
        .expect("Something went wrong!");

    if !child_proc.wait().unwrap().success() {
        println!("Build failed.");
        return;
    }

    let mut child_proc = Command::new("cmd")
        .args(&["/C", "regsvr32 /s target/release/server.dll"])
        .spawn()
        .expect("Something went wrong!");
    if !child_proc.wait().unwrap().success() {
        println!("Failed to register server.dll! Make sure you have administrator rights!");
        return;
    }

    let mut child_proc = Command::new("cmd")
        .args(&["/C", "cargo run --release --package client"])
        .spawn()
        .expect("Something went wrong!");
    if !child_proc.wait().unwrap().success() {
        println!("Execution of client failed.");
        return;
    }
}
