use std::{fs::File, io::Read};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    rom_path: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut file = File::open(args.rom_path)?;
    let mut rom = Vec::new();

    file.read_to_end(&mut rom)?;

    println!("{:?}", rom);

    Ok(())
}
