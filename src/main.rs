use clap::Parser;
use rust_note_search::cli::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = rust_note_search::app::run(cli) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
