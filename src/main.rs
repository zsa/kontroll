mod cli;
#[cfg(feature = "upgrade")]
mod upgrade;

fn main() {
    cli::run();
}
