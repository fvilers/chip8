mod cli;
mod cpu;

use crate::cli::Cli;
use clap::Parser;
use cpu::Cpu;
use std::{fs::File, io::Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut file = File::open(args.rom_path)?;
    let mut rom = Vec::new();

    file.read_to_end(&mut rom)?;

    let mut cpu = Cpu::new(rom);
    cpu.run();

    Ok(())
}
