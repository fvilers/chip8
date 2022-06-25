use clap::Parser;

#[derive(Parser)]
struct Cli {
    rom_path: std::path::PathBuf,
}

fn main() {
    let _args = Cli::parse();
}
