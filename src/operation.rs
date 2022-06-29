pub enum Operation {
    // 0NNN
    CallMachineCodeRoutineAt { address: u16 },

    // 00E0
    ClearScreen,

    // 00EE
    ReturnFromSubroutine,

    // 1NNN
    JumpTo { address: u16 },

    // 2NNN
    CallSubroutineAt { address: u16 },

    // 3XNN
    SkipNextInstructionIfVXEquals { x: u8, value: u8 },

    // 4XNN
    SkipNextInstructionIfVXNotEquals { x: u8, value: u8 },

    // 5XY0
    SkipNextInstructionIfVXEqualsVY { x: u8, y: u8 },

    // 6XNN
    SetVXTo { x: u8, value: u8 },

    // 7XNN
    AddToVX { x: u8, value: u8 },

    // 8XY0
    SetVXToVY { x: u8, y: u8 },

    // 8XY1
    SetVXToVXOrVY { x: u8, y: u8 },

    // 8XY2
    SetVXToVXAndVY { x: u8, y: u8 },

    // 8XY3
    SetVXToVXXorVY { x: u8, y: u8 },

    // 8XY4
    AddVYToVX { x: u8, y: u8 },

    // 8XY5
    SubtractVYFromVX { x: u8, y: u8 },

    // 8XY6
    RightShiftVX { x: u8, y: u8 },

    // 8XY7
    SubtractVXFromVY { x: u8, y: u8 },

    // 8XYE
    LeftShiftVX { x: u8, y: u8 },

    // 9XY0
    SkipNextInstructionIfVXNotEqualsVY { x: u8, y: u8 },

    // ANNN
    SetITo { address: u16 },

    // BNNN
    JumpToPlusV0 { address: u16 },

    // BXNN
    JumpToPlusVX { x: u8, address: u16 },

    // CXNN
    SetVXToVXAndRandomNumber { x: u8, value: u8 },

    // DXYN
    DrawSpriteAt { x: u8, y: u8, height: u8 },

    // EX9E
    SkipNextInstructionIfKeyInVXPressed { x: u8 },

    // EXA1
    SkipNextInstructionIfKeyInVXNotPressed { x: u8 },

    // FX07
    SetVXToDelayTimer { x: u8 },

    // FX0A
    AwaitKeyPress { x: u8 },

    // FX15
    SetDelayTimerToVX { x: u8 },

    // FX18
    SetSoundTimerToVX { x: u8 },

    // FX1E
    AddVXToI { x: u8 },

    // FX29
    SetIToSpriteLocationForCharacterInVX { x: u8 },

    // FX33
    StoreBinaryCodedDecimalOfVX { x: u8 },

    // FX55
    StoreFromV0ToVX { x: u8 },

    // FX65
    FillFromV0ToVX { x: u8 },
}

impl Operation {
    pub fn decode(instruction: u16, super_chip: bool) -> Operation {
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
            (0x08, _, _, 0x06) => Operation::RightShiftVX {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x07) => Operation::SubtractVXFromVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x08, _, _, 0x0e) => Operation::LeftShiftVX {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x09, _, _, 0x00) => Operation::SkipNextInstructionIfVXNotEqualsVY {
                x: nibbles.1,
                y: nibbles.2,
            },
            (0x0A, _, _, _) => Operation::SetITo {
                address: instruction & 0x0FFF,
            },
            (0x0B, _, _, _) => match super_chip {
                false => Operation::JumpToPlusV0 {
                    address: instruction & 0x0FFF,
                },
                true => Operation::JumpToPlusVX {
                    x: nibbles.1,
                    address: instruction & 0x0FFF,
                },
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
