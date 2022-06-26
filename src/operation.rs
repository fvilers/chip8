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
    RightShiftVX { x: u8 },

    // 8XY7
    SubtractVXFromVY { x: u8, y: u8 },

    // 8XYE
    LeftShiftVX { x: u8 },

    // 9XY0
    SkipNextInstructionIfVXNotEqualsVY { x: u8, y: u8 },

    // ANNN
    SetITo { address: u16 },

    // BNNN
    JumpToPlusV0 { address: u16 },

    // CXNN
    SetVXToVXAndRandomNumber { x: u8 },

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
