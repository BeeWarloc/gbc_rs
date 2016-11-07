#[macro_use]
extern crate bitflags;

extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::env;
use std::path::PathBuf;
use std::boxed::Box;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

mod gbc;

use gbc::cart::Cart;
use gbc::cpu::Cpu;
use gbc::ppu::Ppu;
use gbc::spu::Spu;
use gbc::gamepad::Gamepad;
use gbc::interconnect::Interconnect;

fn load_bin(path: &PathBuf) -> Box<[u8]> {
    let mut bytes = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut bytes).unwrap();
    bytes.into_boxed_slice()
}

fn main() {
    let rom_path = PathBuf::from(env::args().nth(1).unwrap());
    let rom_binary = load_bin(&rom_path);

    println!("ROM file name: {:?}", rom_path.file_name().unwrap());
    println!("ROM size: {:?}", rom_binary.len());

    let cart = Cart::new(rom_binary);

    println!("ROM title: {:?}", cart.title());
    println!("ROM type: {:?}", cart.cart_type());
    println!("ROM size: {:?}", cart.rom_size());
    println!("ROM bank count: {:?}", cart.rom_bank_count());
    println!("ROM ram size: {:?}", cart.ram_size());
    println!("ROM ram bank count: {:?}", cart.ram_bank_count());
    println!("ROM destination code: {:?}", cart.destination_code());

    println!("Gameboy type: {:?}", cart.gameboy_type());
    println!("\n");

    // let gb_type = cart.gameboy_type();
    let gb_type = gbc::GameboyType::Dmg;

    let (tx, rx): (Sender<Box<[u8]>>, Receiver<Box<[u8]>>) = mpsc::channel();

    let ppu = Ppu::new(tx.clone());
    let spu = Spu::new();
    let gamepad = Gamepad::new();
    let interconnect = Interconnect::new(gb_type, cart, ppu, spu, gamepad);

    let mut cpu = Cpu::new(gb_type, interconnect);

    let mut cycle_count: u64 = 0;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("GBC_RS", 160 * 6, 144 * 6)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::BGRA8888, 160, 144)
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {

        loop {
            let elapsed_cycles = cpu.step() as u64;
            cycle_count += elapsed_cycles;
            if cycle_count >= 70224 {
                cycle_count = 0;
                break;
            }
        }

        if let Ok(framebuffer) = rx.try_recv() {
            texture.update(None, &framebuffer, 160).unwrap();
        }

        renderer.clear();
        renderer.copy(&texture, None, None).unwrap();
        renderer.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
    }


}
