use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Run as the SUPER-CHIP
    #[clap(short, long, action, default_value_t = false)]
    pub super_chip: bool,

    /// Path to the ROM file
    pub rom_path: std::path::PathBuf,
}
