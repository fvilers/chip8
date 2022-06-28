use crate::{operation::Operation, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::random;

// The memory should be 4 kB (4 kilobytes, ie. 4096 bytes) large.
const RAM_SIZE: u16 = 0x1000;
const VRAM_SIZE: u16 = SCREEN_WIDTH as u16 * SCREEN_HEIGHT as u16;

pub struct Cpu {
    // CHIP-8 has direct access to up to 4 kilobytes of RAM.
    ram: [u8; RAM_SIZE as usize],

    // A program counter which points at the current instruction in memory.
    pc: u16,

    // One 16-bit index register which is used to point at locations in memory.
    i: u16,

    // A stack which is used to call subroutines/functions and return from them.
    stack: Vec<u16>,

    // 16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal. VF is also used as a
    // flag register; many instructions will set it to either 1 or 0 based on some rule.
    v: [u8; 16],

    vram: [u8; VRAM_SIZE as usize],
    vram_changed: bool,
}

impl Cpu {
    pub fn new(rom: Vec<u8>) -> Cpu {
        if rom.len() > RAM_SIZE as usize {
            panic!("ROM size cannot be larger than {} bytes", RAM_SIZE);
        }

        let mut ram = [0x00; RAM_SIZE as usize];
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
            vram: [0x00; VRAM_SIZE as usize],
            vram_changed: false,
        }
    }

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc as usize];
        let low = self.ram[(self.pc + 1) as usize];

        self.pc += 2;

        ((high as u16) << 8) | low as u16
    }

    fn decode(&self, instruction: u16) -> Operation {
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
                address: instruction & 0x0FFF,
            },
            (0x02, _, _, _) => Operation::CallSubroutineAt {
                address: instruction & 0x0FFF,
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
                address: instruction & 0x0FFF,
            },
            (0x0B, _, _, _) => Operation::JumpToPlusV0 {
                address: instruction & 0x0FFF,
            },
            (0x0C, _, _, _) => Operation::SetVXToVXAndRandomNumber {
                x: nibbles.1,
                value: (instruction & 0x00FF) as u8,
            },
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
                address: instruction & 0x0FFF,
            },

            _ => panic!("Unsupported instruction {:04x}", instruction),
        }
    }

    fn execute(&mut self, operation: Operation) {
        match operation {
            // In the original CHIP-8 interpreters, this would pause execution of the CHIP-8 program and call a
            // subroutine written in machine language at address NNN instead. This routine would be written in the
            // machine language of the computer's CPU; on the original COSMAC VIP and the ETI-660, this was 1802
            // machine code, and on the DREAM 6800, M6800 code. Unless you're making an emulator for either of those
            // computers, skip this one.
            Operation::CallMachineCodeRoutineAt { address: _ } => {}

            // Clear the display, turning all pixels off to 0.
            Operation::ClearScreen => {
                for i in 0..self.vram.len() {
                    self.vram[i] = 0;
                }

                self.vram_changed = true;
            }

            // Return from a subroutine by removing the last address from the stack.
            Operation::ReturnFromSubroutine => {
                self.pc = self.stack.pop().expect("Cannot pop() from an empty stack");
            }

            // Simply set PC to address, causing the program to jump to that memory location.
            Operation::JumpTo { address } => {
                self.pc = address;
            }

            // Calls the subroutine at memory location.
            Operation::CallSubroutineAt { address } => {
                self.stack.push(self.pc);
                self.pc = address;
            }

            // Skip one instruction if the value in VX is equal to value.
            Operation::SkipNextInstructionIfVXEquals { x, value } => {
                if self.v[x as usize] == value {
                    self.pc += 2;
                }
            }

            // Skip one instruction if the value in VX is not equal to NN.
            Operation::SkipNextInstructionIfVXNotEquals { x, value } => {
                if self.v[x as usize] != value {
                    self.pc += 2;
                }
            }

            // Skips if the values in VX and VY are equal.
            Operation::SkipNextInstructionIfVXEqualsVY { x, y } => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }

            // Set the register VX to the value NN.
            Operation::SetVXTo { x, value } => {
                self.v[x as usize] = value;
            }

            // Add the value NN to VX.
            // Note that on most other systems, and even in some of the other CHIP-8 instructions, this would set the
            // carry flag if the result overflowed 8 bits. For this instruction, this is not the case.
            Operation::AddToVX { x, value } => {
                let (result, _overflow) = self.v[x as usize].overflowing_add(value);
                self.v[x as usize] = result;
            }

            // VX is set to the value of VY.
            Operation::SetVXToVY { x, y } => {
                self.v[x as usize] = self.v[y as usize];
            }

            // VX is set to the bitwise/binary logical disjunction (OR) of VX and VY.
            Operation::SetVXToVXOrVY { x, y } => {
                self.v[x as usize] |= self.v[y as usize];
            }

            // VX is set to the bitwise/binary logical conjunction (AND) of VX and VY.
            Operation::SetVXToVXAndVY { x, y } => {
                self.v[x as usize] &= self.v[y as usize];
            }

            // VX is set to the bitwise/binary exclusive OR (XOR) of VX and VY.
            Operation::SetVXToVXXorVY { x, y } => {
                self.v[x as usize] ^= self.v[y as usize];
            }

            // VX is set to the value of VX plus the value of VY.
            Operation::AddVYToVX { x, y } => {
                // Unlike 7XNN, this addition will affect the carry flag. If the result is larger than 255 (and thus
                // overflows the 8-bit register VX), the flag register VF is set to 1. If it doesn't overflow, VF is set
                // to 0.
                let (result, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[x as usize] = result;
                self.v[0x0F] = match overflow {
                    true => 1,
                    false => 0,
                };
            }

            // Sets VX to the result of VX - VY. This subtraction will also affect the carry flag, but note that
            // it's opposite from what you might think. If the minuend (the first operand) is larger than the subtrahend
            // (second operand), VF will be set to 1. If the subtrahend is larger, and we "underflow" the result, VF is
            // set to 0.
            Operation::SubtractVYFromVX { x, y } => {
                let (result, overflow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[x as usize] = result;
                self.v[0x0F] = match overflow {
                    true => 0,
                    false => 1,
                };
            }

            // In the CHIP-8 interpreter for the original COSMAC VIP, this instruction did the following: it put
            // the value of VY into VX, and then shifted the value in VX 1 bit to the right (8XY6) or left (8XYE). VY
            // was not affected, but the flag register VF would be set to the bit that was shifted out.
            // However, starting with CHIP-48 and SUPER-CHIP in the early 1990s, these instructions were changed so that
            // they shifted VX in place, and ignored the Y completely.
            Operation::RightShiftVX { x } => {
                // TODO: handle optional behavior for SUPER-CHIP (set VX to the value of VY)

                self.v[0x0F] = self.v[x as usize] & 1;
                self.v[x as usize] >>= 1;
            }
            Operation::LeftShiftVX { x } => {
                // TODO: handle optional behavior for SUPER-CHIP (set VX to the value of VY)

                self.v[0x0F] = self.v[x as usize] >> 3;
                self.v[x as usize] <<= 1;
            }

            // Sets VX to the result of VY - VX. This subtraction will also affect the carry flag the same way than
            // 8XY5.
            Operation::SubtractVXFromVY { x, y } => {
                let (result, overflow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[x as usize] = result;
                self.v[0x0F] = match overflow {
                    true => 0,
                    false => 1,
                };
            }

            // Skips if the values in VX and VY are not equal.
            Operation::SkipNextInstructionIfVXNotEqualsVY { x, y } => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc = self.pc + 2;
                }
            }

            // This sets the index register I to the value address.
            Operation::SetITo { address } => {
                self.i = address;
            }

            // In the original COSMAC VIP interpreter, this instruction jumped to the address NNN plus the value
            // in the register V0. This was mainly used for "jump tables", to quickly be able to jump to different
            // subroutines based on some input.
            // Starting with CHIP-48 and SUPER-CHIP, it was (probably unintentionally) changed to work as BXNN: It will
            // jump to the address XNN, plus the value in the register VX. So the instruction B220 will jump to address
            // 220 plus the value in the register V2.
            Operation::JumpToPlusV0 { address } => {
                // TODO: handle quirk mode for SUPER-CHIP

                self.pc = address + self.v[0] as u16;
            }

            // This instruction generates a random number, binary ANDs it with the value NN, and puts the result in VX.
            Operation::SetVXToVXAndRandomNumber { x, value } => {
                let random: u8 = random();
                self.v[x as usize] = random & value;
            }

            // Draw an N pixels tall sprite from the memory location that the I index register is holding to the screen,
            // at the horizontal X coordinate in VX and the Y coordinate in VY.
            Operation::DrawSpriteAt { x, y, height } => {
                let mut y = self.v[y as usize] % SCREEN_HEIGHT;

                self.v[0x0F] = 0;

                for row in 0..height {
                    // The starting position of the sprite will wrap. In other words, an X coordinate of 5 is the same
                    //  as an X of 68 (since the screen is 64 pixels wide)
                    let mut x = self.v[x as usize] % SCREEN_WIDTH;
                    let address = self.i + row as u16;
                    let sprite_row = self.ram[address as usize];

                    for bit in 0..8 {
                        let pixel = (sprite_row >> (7 - bit)) & 1;
                        let screen_position = (y as u16 * SCREEN_HEIGHT as u16) + x as u16;

                        // If the current pixel in the sprite row is on and the pixel at coordinates X,Y on the screen
                        // is also on, turn off the pixel and set VF to 1. Or if the current pixel in the sprite row is
                        // on and the screen pixel is not, draw the pixel at the X and Y coordinates.
                        self.v[0x0f] |= pixel & self.vram[screen_position as usize];
                        self.vram[screen_position as usize] ^= pixel;

                        if x == SCREEN_WIDTH - 1 {
                            break;
                        }
                        x += 1;
                    }

                    if y == SCREEN_HEIGHT - 1 {
                        break;
                    }
                    y += 1;
                }

                self.vram_changed = true;
            }

            // Skip the following instruction based on a condition. These skip based on whether the player is currently
            // pressing a key or not.
            Operation::SkipNextInstructionIfKeyInVXPressed { x: _ } => {
                // TODO:
                unimplemented!()
            }
            Operation::SkipNextInstructionIfKeyInVXNotPressed { x: _ } => {
                // TODO:
                unimplemented!()
            }

            // Sets VX to the current value of the delay timer.
            Operation::SetVXToDelayTimer { x: _ } => {
                //TODO:
                unimplemented!()
            }

            // This instruction "blocks"; it stops executing instructions and waits for key input (or loops forever,
            // unless a key is pressed).
            // As we increment PC after fetching each instruction, then it should be decremented again here unless a key
            // is pressed. Otherwise, PC should simply not be incremented.
            // Although this instruction stops the program from executing further instructions, the timers (delay timer
            // and sound timer) should still be decreased while it's waiting.
            // If a key is pressed while this instruction is waiting for input, its hexadecimal value will be put in VX
            // and execution continues.
            Operation::AwaitKeyPress { x: _ } => {
                //TODO:
                unimplemented!()
            }

            // Sets the delay timer to the value in VX.
            Operation::SetDelayTimerToVX { x: _ } => {
                //TODO:
                unimplemented!()
            }

            // Sets the sound timer to the value in VX.
            Operation::SetSoundTimerToVX { x: _ } => {
                // TODO:
                unimplemented!()
            }

            // The index register I will get the value in VX added to it.
            // Unlike other arithmetic instructions, this did not affect VF on overflow on the original COSMAC VIP.
            // However, it seems that some interpreters set VF to 1 if I “overflows” from 0FFF to above 1000 (outside
            // the normal addressing range). This wasn't the case on the original COSMAC VIP, at least, but apparently
            // the CHIP-8 interpreter for Amiga behaved this way. At least one known game, Spacefight 2091!, relies on
            // this behavior.
            Operation::AddVXToI { x } => {
                // TODO: see comment above if we want to support that case
                self.i += self.v[x as usize] as u16;
            }

            // The index register I is set to the address of the hexadecimal character in VX. An 8-bit register can hold
            // two hexadecimal numbers, but this would only point to one character. The original COSMAC VIP
            // interpreter just took the last nibble of VX and used that as the character.
            Operation::SetIToSpriteLocationForCharacterInVX { x: _ } => {
                //TODO:
                unimplemented!()
            }

            // It takes the number in VX (which is one byte, so it can be any number from 0 to 255) and converts it to
            // three decimal digits, storing these digits in memory at the address in the index register I.
            Operation::StoreBinaryCodedDecimalOfVX { x } => {
                let n = self.v[x as usize];
                let i = self.i as usize;

                self.ram[i] = n / 100;
                self.ram[i + 1] = (n % 100) / 10;
                self.ram[i + 2] = n % 10;
            }

            // The value of each variable register from V0 to VX inclusive (if X is 0, then only V0) will be stored in
            // successive memory addresses, starting with the one that's stored in I.
            // The original CHIP-8 interpreter for the COSMAC VIP actually incremented the I register while it worked.
            // Each time it stored or loaded one register, it incremented I. After the instruction was finished, I would
            // be set to the new value I + X + 1.
            // However, modern interpreters (starting with CHIP48 and SUPER-CHIP in the early 90s) used a temporary
            // variable for indexing, so when the instruction was finished, I would still hold the same value as it did
            // before.
            Operation::StoreFromV0ToVX { x } => {
                // TODO: handle optional behavior for SUPER-CHIP

                for n in 0..=x as usize {
                    self.ram[n] = self.v[n];
                }

                self.i += 1 + x as u16;
            }

            // Does the same thing than FX55, except that it takes the value stored at the memory addresses and loads
            // them into the variable registers instead.
            Operation::FillFromV0ToVX { x } => {
                for n in 0..=x as usize {
                    self.v[n] = self.ram[n];
                }

                self.i += 1 + x as u16;
            }
        }
    }

    // An emulator's main task is simple. It runs in an infinite loop, and does these three tasks in succession.
    pub fn tick(&mut self) -> bool {
        // Fetch the instruction from memory at the current PC.
        let instruction = self.fetch();

        // Decode the instruction to find out what the emulator should do.
        let operation = self.decode(instruction);

        // Execute the instruction and do what it tells you.
        self.execute(operation);

        // Let the CPU consumer know if VRAM has changed.
        self.vram_changed
    }

    pub fn draw(&mut self, screen: &mut [u8]) {
        for (p, pixel) in self.vram.iter().zip(screen.chunks_exact_mut(4)) {
            let color = match p {
                0 => 0x00,
                1 => 0xFF,
                _ => panic!("Invalid pixel value"),
            };
            pixel[0] = color; // Red
            pixel[1] = color; // Green
            pixel[2] = color; // Blue
            pixel[3] = 0xFF; // Alpha channel
        }

        self.vram_changed = false;
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
