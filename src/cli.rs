use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub rom_path: std::path::PathBuf,
}
