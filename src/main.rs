mod cli;
mod cpu;
mod font;
mod key_mapping;
mod operation;

use crate::cli::Cli;
use crate::cpu::Cpu;
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use std::{fs::File, io::Read};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

// The display is 64 pixels wide and 32 pixels tall.
// TODO: or 128 x 64 for SUPER-CHIP.
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut file = File::open(&args.rom_path)?;
    let mut rom = Vec::new();

    file.read_to_end(&mut rom)?;

    let mut cpu = Cpu::new(rom, args.super_chip);
    let file_name = args.rom_path.file_name().unwrap().to_str().unwrap();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);

        WindowBuilder::new()
            .with_title([&file_name, "CHIP-8"].join(" - "))
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        Pixels::new(SCREEN_WIDTH.into(), SCREEN_HEIGHT.into(), surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't dispatched any events. This is
        // ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let frame = pixels.get_frame();

                cpu.draw(frame);
                pixels.render().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => {
                let (should_redraw, should_beep) = cpu.tick();

                if should_redraw {
                    window.request_redraw();
                }

                if should_beep {
                    beep(440, 10);
                }
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            for (vkc, key) in key_mapping::KEY_MAPPING {
                if input.key_pressed(vkc) {
                    cpu.press_key(key);
                    break;
                }

                if input.key_released(vkc) {
                    cpu.release_key();
                    break;
                }
            }
        }
    });
}

#[cfg(windows)]
fn beep(frequency: u32, duration: u32) {
    use winapi::um::utilapiset;

    unsafe {
        utilapiset::Beep(frequency, duration);
    }
}
#[cfg(not(windows))]
fn beep(frequency: u32, duration: u32) {
    // TODO: implement beep for other platforms
    println!("beep() is not supported")
}
