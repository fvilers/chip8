mod cli;
mod cpu;
mod operation;

use crate::cli::Cli;
use crate::cpu::Cpu;
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use std::{fs::File, io::Read};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// The display is 64 pixels wide and 32 pixels tall.
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut file = File::open(&args.rom_path)?;
    let mut rom = Vec::new();

    file.read_to_end(&mut rom)?;

    let mut cpu = Cpu::new(rom);
    let file_name = args.rom_path.file_name().unwrap().to_str().unwrap();

    let event_loop = EventLoop::new();
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
                if cpu.tick() {
                    window.request_redraw();
                }
            }
        }
    });
}
