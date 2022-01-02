mod chip_8;

use std::collections::HashMap;
use std::env;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use std::time::Duration;
use sdl2::render::TextureAccess;
use crate::chip_8::KeyboardState;

pub fn main() {
    const SCALE: u32 = 6;

    let args: Vec<String> = env::args().collect();

    let key_map = HashMap::from([
        (Scancode::Num1, 0x1), (Scancode::Num2, 0x2), (Scancode::Num3, 0x3), (Scancode::Num4, 0xC),
        (Scancode::Q, 0x4), (Scancode::W, 0x5), (Scancode::E, 0x6), (Scancode::R, 0xD),
        (Scancode::A, 0x7), (Scancode::S, 0x8), (Scancode::D, 0x9), (Scancode::F, 0xE),
        (Scancode::Z, 0xA), (Scancode::X, 0x0), (Scancode::C, 0xB), (Scancode::V, 0xF),
    ]);

    let mut chip8 = chip_8::Chip8::new(std::fs::read(args.get(1).expect("path to ROM required")).unwrap().as_slice());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip-8", chip_8::SCREEN_WIDTH as u32 * SCALE, chip_8::SCREEN_HEIGHT as u32 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture(PixelFormatEnum::RGB332, TextureAccess::Streaming, chip_8::SCREEN_WIDTH as u32, chip_8::SCREEN_HEIGHT as u32).unwrap();

    let mut keyboard_state: KeyboardState = [false; 16];

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { scancode, .. } => {
                    if let Some(k) = key_map.get(&scancode.unwrap()) {
                        keyboard_state[*k] = true;
                    }
                }
                Event::KeyUp { scancode, .. } => {
                    if let Some(k) = key_map.get(&scancode.unwrap()) {
                        keyboard_state[*k] = false;
                    }
                }
                _ => {}
            }
        }

        chip8.frame(&keyboard_state);

        texture.update(None, &chip8.screen, chip_8::SCREEN_WIDTH as usize);
        canvas.copy(&texture, None, None);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}