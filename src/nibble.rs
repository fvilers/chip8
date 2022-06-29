pub trait Nibble {
    fn get_nibbles(&self) -> (u8, u8, u8, u8);
}

impl Nibble for u16 {
    fn get_nibbles(&self) -> (u8, u8, u8, u8) {
        (
            ((self & 0xF000) >> 0xC) as u8,
            ((self & 0x0F00) >> 0x8) as u8,
            ((self & 0x00F0) >> 0x4) as u8,
            (self & 0x000F) as u8,
        )
    }
}
