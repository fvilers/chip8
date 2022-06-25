pub struct Cpu {
    // CHIP-8 has direct access to up to 4 kilobytes of RAM
    ram: [u8; 0x1000],

    // A program counter which points at the current instruction in memory
    pc: u16,

    // One 16-bit index register which is used to point at locations in memory
    i: u16,

    // A stack which is used to call subroutines/functions and return from them
    stack: Vec<u16>,

    // 16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal
    // VF is also used as a flag register; many instructions will set it to either 1 or 0 based on some rule
    v: [u8; 16],
}

impl Cpu {
    pub fn new(rom: Vec<u8>) -> Cpu {
        // TODO: ensure rom size is not larger than ram size

        let mut ram = [0x00; 0x1000];
        let mut i = 0x200;

        for byte in rom {
            ram[i] = byte;
            i += 1;
        }

        Cpu {
            ram,
            pc: 0x200,
            i: 0x00,
            stack: Vec::new(),
            v: [0x00; 16],
        }
    }

    pub fn run(&self) {
        // An emulator’s main task is simple. It runs in an infinite loop, and does these three tasks in succession
        loop {
            // Fetch the instruction from memory at the current PC

            // Decode the instruction to find out what the emulator should do

            // Execute the instruction and do what it tells you
        }
    }
}
