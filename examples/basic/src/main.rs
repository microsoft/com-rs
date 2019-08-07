use std::process::{Command, Stdio};
use std::io::{self, Write};

fn main() {
    let mut child_proc = Command::new("cmd")
        .args(&["/C", "cls && cargo build --all --release"])
        .spawn()
        .expect("Something went wrong!");
    child_proc.wait().unwrap();

    let mut child_proc = Command::new("cmd")
        .args(&["/C", "regsvr32 /s target/release/server.dll"])
        .spawn()
        .expect("Something went wrong!");
    child_proc.wait().unwrap();

    let mut child_proc = Command::new("cmd")
        .args(&["/C", "cargo run --release --package client"])
        .spawn()
        .expect("Something went wrong!");
    child_proc.wait().unwrap();
}