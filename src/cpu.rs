use std::time::{Duration, Instant};

use crate::{font::FONT, operation::Operation, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::random;

// The first CHIP-8 interpreter (on the COSMAC VIP computer) was also located in RAM, from address 000 to 1FF. It would
//  expect a CHIP-8 program to be loaded into memory after it, starting at address 200.
const ROM_ADDRESS: u16 = 0x200;

// The memory should be 4 kB (4 kilobytes, ie. 4096 bytes) large.
const RAM_SIZE: usize = 0x1000;
const VRAM_SIZE: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;

// Original interpreters had limited space on the stack; usually at least 16 two-byte entries.
const STACK_SIZE: usize = 16;

// Timers should be decremented by one 60 times per second (ie. at 60 Hz). This is independent of the speed of the
// fetch/decode/execute loop.
const TIMER_FREQUENCY: Duration = Duration::from_micros(1000000 / 60); // 60 times per second

// We should store the font data in memory, because games will draw these characters like regular sprites: They set the
// index register I to the character's memory location and then draw it. There's a special instruction for setting I to
// a character's address, so we can choose where to put it. Anywhere in the first 512 bytes (000–1FF) is fine. For some
// reason, it's become popular to put it at 050–09F, so you can follow that convention if you want.
const FONT_ADDRESS: u16 = 0x050;

pub struct Cpu {
    // CHIP-8 has direct access to up to 4 kilobytes of RAM.
    ram: [u8; RAM_SIZE as usize],

    // A program counter which points at the current instruction in memory.
    pc: u16,

    // One 16-bit index register which is used to point at locations in memory.
    i: u16,

    // A stack which is used to call subroutines/functions and return from them.
    stack: Vec<u16>,

    // An 8-bit delay timer which is decremented at a rate of 60 Hz (60 times per second) until it reaches 0
    delay_timer: u8,

    // An 8-bit sound timer which functions like the delay timer, but which also gives off a beeping sound as long as
    // it’s not 0
    sound_timer: u8,

    // 16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal. VF is also used as a
    // flag register; many instructions will set it to either 1 or 0 based on some rule.
    v: [u8; 16],

    vram: [u8; VRAM_SIZE],
    vram_changed: bool,

    // Flag set to `true` if the SUPER-CHIP behavior is enabled
    super_chip: bool,

    last_tick: Instant,
    key_held: Option<u8>,
}

impl Cpu {
    pub fn new(rom: Vec<u8>, super_chip: bool) -> Cpu {
        if rom.len() > RAM_SIZE {
            panic!("ROM size cannot be larger than {} bytes", RAM_SIZE);
        }

        let mut ram = [0x00; RAM_SIZE];
        let mut i = ROM_ADDRESS as usize;

        // Copy the ROM to the RAM
        for byte in rom {
            ram[i] = byte;
            i += 1;
        }

        // Copy the font to the RAM
        i = FONT_ADDRESS as usize;
        for byte in FONT {
            ram[i] = byte;
            i += 1;
        }

        Cpu {
            ram,
            pc: ROM_ADDRESS,
            i: 0x00,
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0x00,
            sound_timer: 0x00,
            v: [0x00; 16],
            vram: [0x00; VRAM_SIZE],
            vram_changed: false,
            super_chip,
            last_tick: Instant::now(),
            key_held: Option::None,
        }
    }

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc as usize];
        let low = self.ram[(self.pc + 1) as usize];

        self.pc += 2;

        ((high as u16) << 8) | low as u16
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
                if self.stack.len() == STACK_SIZE {
                    panic!("Stack size limit of {} reached.", STACK_SIZE);
                }

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
            Operation::RightShiftVX { x, y } => {
                if !self.super_chip {
                    self.v[x as usize] = self.v[y as usize];
                }

                self.v[0x0F] = self.v[x as usize] & 1;
                self.v[x as usize] >>= 1;
            }
            Operation::LeftShiftVX { x, y } => {
                if !self.super_chip {
                    self.v[x as usize] = self.v[y as usize];
                }

                self.v[0x0F] = (self.v[x as usize] & 0x80) >> 7;
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
                self.pc = address + self.v[0] as u16;
            }

            Operation::JumpToPlusVX { x, address } => {
                self.pc = address + self.v[x as usize] as u16;
            }

            // This instruction generates a random number, binary ANDs it with the value NN, and puts the result in VX.
            Operation::SetVXToVXAndRandomNumber { x, value } => {
                let random: u8 = random();
                self.v[x as usize] = random & value;
            }

            // Draw an N pixels tall sprite from the memory location that the I index register is holding to the screen,
            // at the horizontal X coordinate in VX and the Y coordinate in VY.
            Operation::DrawSpriteAt { x, y, height } => {
                // The starting position of the sprite will wrap. In other words, an X coordinate of 5 is the same as an
                //  X of 68 (since the screen is 64 pixels wide)
                let x = self.v[x as usize] % SCREEN_WIDTH;
                let y = self.v[y as usize] % SCREEN_HEIGHT;

                self.v[0x0F] = 0;

                for row in 0..height {
                    let address = self.i + row as u16;
                    let sprite_row = self.ram[address as usize];
                    let coords_y = y + row;

                    for col in 0..8 {
                        let pixel = (sprite_row >> (7 - col)) & 1;
                        let coords_x = x + col;
                        let screen_position =
                            coords_x as u16 + (coords_y as u16 * SCREEN_WIDTH as u16);

                        // If the current pixel in the sprite row is on and the pixel at coordinates X,Y on the screen
                        // is also on, turn off the pixel and set VF to 1. Or if the current pixel in the sprite row is
                        // on and the screen pixel is not, draw the pixel at the X and Y coordinates.
                        // if pixel != 0 {
                        self.v[0x0f] |= pixel & self.vram[screen_position as usize];
                        self.vram[screen_position as usize] ^= pixel;

                        if coords_x == SCREEN_WIDTH - 1 {
                            break;
                        }
                    }

                    if coords_y == SCREEN_HEIGHT - 1 {
                        break;
                    }
                }

                self.vram_changed = true;
            }

            // Skip the following instruction based on a condition. These skip based on whether the player is currently
            // pressing a key or not.
            Operation::SkipNextInstructionIfKeyInVXPressed { x } => {
                self.pc += match self.key_held {
                    Some(key) if key == self.v[x as usize] => 2,
                    _ => 0,
                }
            }
            Operation::SkipNextInstructionIfKeyInVXNotPressed { x } => {
                self.pc += match self.key_held {
                    Some(key) if key == self.v[x as usize] => 0,
                    _ => 2,
                }
            }

            // Sets VX to the current value of the delay timer.
            Operation::SetVXToDelayTimer { x } => {
                self.v[x as usize] = self.delay_timer;
            }

            // This instruction "blocks"; it stops executing instructions and waits for key input (or loops forever,
            // unless a key is pressed).
            // As we increment PC after fetching each instruction, then it should be decremented again here unless a key
            // is pressed. Otherwise, PC should simply not be incremented.
            // Although this instruction stops the program from executing further instructions, the timers (delay timer
            // and sound timer) should still be decreased while it's waiting.
            // If a key is pressed while this instruction is waiting for input, its hexadecimal value will be put in VX
            // and execution continues.
            Operation::AwaitKeyPress { x } => {
                self.pc -= match self.key_held {
                    Some(key) => {
                        self.v[x as usize] = key;
                        0
                    }
                    None => 2,
                }
            }

            // Sets the delay timer to the value in VX.
            Operation::SetDelayTimerToVX { x } => {
                self.delay_timer = self.v[x as usize];
            }

            // Sets the sound timer to the value in VX.
            Operation::SetSoundTimerToVX { x } => {
                self.sound_timer = self.v[x as usize];
            }

            // The index register I will get the value in VX added to it.
            // Unlike other arithmetic instructions, this did not affect VF on overflow on the original COSMAC VIP.
            // However, it seems that some interpreters set VF to 1 if I "overflows" from 0FFF to above 1000 (outside
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
            Operation::SetIToSpriteLocationForCharacterInVX { x } => {
                let offset = self.v[x as usize] as u16 * 5;
                self.i = FONT_ADDRESS + offset as u16;
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
            Operation::StoreFromV0ToVX { x } => match self.super_chip {
                false => {
                    for i in 0..=x {
                        self.ram[self.i as usize] = self.v[i as usize];
                        self.i += 1;
                    }
                }
                true => {
                    for i in 0..=x {
                        let address = self.i + i as u16;
                        self.ram[address as usize] = self.v[i as usize];
                    }
                }
            },

            // Does the same thing than FX55, except that it takes the value stored at the memory addresses and loads
            // them into the variable registers instead.
            Operation::FillFromV0ToVX { x } => match self.super_chip {
                false => {
                    for i in 0..=x {
                        self.v[i as usize] = self.ram[self.i as usize];
                        self.i += 1;
                    }
                }
                true => {
                    for i in 0..=x {
                        let address = self.i + i as u16;
                        self.v[i as usize] = self.ram[address as usize];
                    }
                }
            },
        }
    }

    // An emulator's main task is simple. It runs in an infinite loop, and does these three tasks in succession.
    pub fn tick(&mut self) -> (bool, bool) {
        // Fetch the instruction from memory at the current PC.
        let instruction = self.fetch();

        // Decode the instruction to find out what the emulator should do.
        let operation = Operation::decode(instruction, self.super_chip);

        // Execute the instruction and do what it tells you.
        self.execute(operation);

        // Decrement timers only if enough time has passed
        let mut should_beep = false;

        if self.last_tick.elapsed().gt(&TIMER_FREQUENCY) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                should_beep = true;
                self.sound_timer -= 1;
            }

            // Tick
            self.last_tick = Instant::now();
        }

        // Let the CPU consumer know if VRAM has changed and if it needs to beep
        (self.vram_changed, should_beep)
    }

    pub fn draw(&mut self, screen: &mut [u8]) {
        for (p, pixel) in self.vram.iter().zip(screen.chunks_exact_mut(4)) {
            pixel[0] = 0xFF; // Red
            pixel[1] = 0xFF; // Green
            pixel[2] = 0xFF; // Blue
            pixel[3] = match p {
                0 => 0x00,
                1 => 0xFF,
                _ => panic!("Invalid VRAM value ({})", p),
            }; // Alpha channel
        }

        self.vram_changed = false;
    }

    pub fn press_key(&mut self, value: u8) {
        self.key_held = Option::Some(value);
    }

    pub fn release_key(&mut self) {
        self.key_held = Option::None;
    }
}
