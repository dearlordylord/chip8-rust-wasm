mod cpu;
mod screen;
mod console_screen;
mod macros;

use std::thread;
use std::fs::File;
use std::io::Read;
use std::{println};
use cpu::CPU;
use crate::console_screen::ConsoleScreen;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let program = read_rom();
    let mut cpu = CPU::new(Box::new(ConsoleScreen::new()));
    cpu.load_program(program);
    cpu.run().await;
    Ok(())
}

fn read_rom() -> Vec<u8> {
    let mut file = File::open("BLINKY").unwrap();
    let mut r: Vec<u8> = Vec::new();
    file.read_to_end(&mut r).unwrap();
    return r;
}