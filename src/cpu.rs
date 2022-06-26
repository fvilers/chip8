use crate::operation::Operation;

pub struct Cpu {
    // CHIP-8 has direct access to up to 4 kilobytes of RAM.
    ram: [u8; 0x1000],

    // A program counter which points at the current instruction in memory.
    pc: u16,

    // One 16-bit index register which is used to point at locations in memory.
    i: u16,

    // A stack which is used to call subroutines/functions and return from them.
    stack: Vec<u16>,

    // 16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal. VF is also used as a
    // flag register; many instructions will set it to either 1 or 0 based on some rule.
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

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc as usize];
        let low = self.ram[(self.pc + 1) as usize];

        self.pc += 2;

        ((high as u16) << 8) | low as u16
    }

    fn decode(&mut self, instruction: u16) -> Operation {
        // CHIP-8 instructions are divided into broad categories by the first "nibble", or "half-byte", which is the
        // first hexadecimal number. Although every instruction will have a first nibble that tells you what kind of
        // instruction it is, the rest of the nibbles will have different meanings.

        // To differentiate these meanings, we usually call them different things, but all of them can be any
        // hexadecimal number from 0 to F:
        // - X: The second nibble. Used to look up one of the 16 registers (VX) from V0 through VF.
        // - Y: The third nibble. Also used to look up one of the 16 registers (VY) from V0 through VF.
        // - N: The fourth nibble. A 4-bit number.
        // - NN: The second byte (third and fourth nibbles). An 8-bit immediate number.
        // - NNN: The second, third and fourth nibbles. A 12-bit immediate memory address.
        let nibbles = get_nibbles(&instruction);

        match nibbles {
            (0x00, 0x00, 0x0E, 0x00) => Operation::ClearScreen,
            (0x01, _, _, _) => Operation::JumpTo {
                address: instruction & 0x0FF,
            },
            (0x02, _, _, _) => Operation::CallSubroutineAt {
                address: instruction & 0x0FF,
            },
            (0x00, 0x00, 0x0e, 0x0E) => Operation::ReturnFromSubroutine,
            (0x03, _, _, _) => Operation::SkipNextInstructionIfVXEquals {
                x: nibbles.1,
                value: (instruction & 0x00FF) as u8,
            },
            (0x04, _, _, _) => Operation::SkipNextInstructionIfVXNotEquals {
                x: nibbles.1,
                value: (instruction & 0x00FF) as u8,
            },
            (0x05, _, _, 0x00) => Operation::SkipNextInstructionIfVXEqualsVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x06, _, _, _) => Operation::SetVXTo {
                x: nibbles.1,
                value: (instruction & 0x00FF) as u8,
            },
            (0x07, _, _, _) => Operation::AddToVX {
                x: nibbles.1,
                value: (instruction & 0x00FF) as u8,
            },
            (0x08, _, _, 0x00) => Operation::SetVXToVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x01) => Operation::SetVXToVXOrVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x02) => Operation::SetVXToVXAndVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x03) => Operation::SetVXToVXXorVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x04) => Operation::AddVYToVX {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x05) => Operation::SubtractVYFromVX {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x06) => Operation::RightShiftVX { x: nibbles.1 },
            (0x08, _, _, 0x07) => Operation::SubtractVXFromVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x0e) => Operation::LeftShiftVX { x: nibbles.1 },
            (0x09, _, _, 0x00) => Operation::SkipNextInstructionIfVXNotEqualsVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x0A, _, _, _) => Operation::SetITo {
                address: instruction & 0x0FF,
            },
            (0x0B, _, _, _) => Operation::JumpToPlusV0 {
                address: instruction & 0x0FF,
            },
            (0x0C, _, _, _) => Operation::SetVXToVXAndRandomNumber { x: nibbles.1 },
            (0x0D, _, _, _) => Operation::DrawSpriteAt {
                x: nibbles.1,
                y: nibbles.2,
                height: nibbles.3,
            },
            (0x0E, _, 0x09, 0x0E) => {
                Operation::SkipNextInstructionIfKeyInVXPressed { x: nibbles.1 }
            }
            (0x0E, _, 0x0A, 0x01) => {
                Operation::SkipNextInstructionIfKeyInVXNotPressed { x: nibbles.1 }
            }
            (0x0F, _, 0x00, 0x07) => Operation::SetVXToDelayTimer { x: nibbles.1 },
            (0x0F, _, 0x00, 0xA) => Operation::AwaitKeyPress { x: nibbles.1 },
            (0x0F, _, 0x01, 0x05) => Operation::SetDelayTimerToVX { x: nibbles.1 },
            (0x0F, _, 0x01, 0x08) => Operation::SetSoundTimerToVX { x: nibbles.1 },
            (0x0F, _, 0x01, 0x0E) => Operation::AddVXToI { x: nibbles.1 },
            (0x0F, _, 0x02, 0x09) => {
                Operation::SetIToSpriteLocationForCharacterInVX { x: nibbles.1 }
            }
            (0x0F, _, 0x03, 0x03) => Operation::StoreBinaryCodedDecimalOfVX { x: nibbles.1 },
            (0x0F, _, 0x05, 0x05) => Operation::StoreFromV0ToVX { x: nibbles.1 },
            (0x0F, _, 0x06, 0x05) => Operation::FillFromV0ToVX { x: nibbles.1 },

            // Leave this arm as the last one as it could match any 0x00 opcode
            (0x00, _, _, _) => Operation::CallMachineCodeRoutineAt {
                address: instruction & 0x0FF,
            },

            _ => panic!("Unsupported instruction {:04x}", instruction),
        }
    }

    pub fn run(&mut self) {
        // An emulator's main task is simple. It runs in an infinite loop, and does these three tasks in succession.
        loop {
            // Fetch the instruction from memory at the current PC.
            let instruction = self.fetch();

            // Decode the instruction to find out what the emulator should do.
            let operation = self.decode(instruction);

            // Execute the instruction and do what it tells you.
        }
    }
}

// TODO: use a Nibbles new type that doesn't allocates memory, but instead gives a "view" into the nibbles of a slice.
fn get_nibbles(value: &u16) -> (u8, u8, u8, u8) {
    (
        ((value & 0xF000) >> 0xC) as u8,
        ((value & 0x0F00) >> 0x8) as u8,
        ((value & 0x00F0) >> 0x4) as u8,
        (value & 0x000F) as u8,
    )
}
