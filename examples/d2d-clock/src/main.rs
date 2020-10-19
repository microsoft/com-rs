#[cfg(windows)]
mod clock;
fn main() {
    #[cfg(windows)]
    clock::run
}
