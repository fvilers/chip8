use winit::event::VirtualKeyCode;

pub const KEY_MAPPING: [(VirtualKeyCode, u8); 16] = [
    (VirtualKeyCode::Key1, 0x01),
    (VirtualKeyCode::Key2, 0x02),
    (VirtualKeyCode::Key3, 0x03),
    (VirtualKeyCode::Key4, 0x0C),
    (VirtualKeyCode::A, 0x04),
    (VirtualKeyCode::Z, 0x05),
    (VirtualKeyCode::E, 0x06),
    (VirtualKeyCode::R, 0x0D),
    (VirtualKeyCode::Q, 0x07),
    (VirtualKeyCode::S, 0x08),
    (VirtualKeyCode::D, 0x09),
    (VirtualKeyCode::F, 0x0E),
    (VirtualKeyCode::W, 0x0A),
    (VirtualKeyCode::X, 0x00),
    (VirtualKeyCode::C, 0x0B),
    (VirtualKeyCode::V, 0x0F),
];
