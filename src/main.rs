mod cpu;
mod screen;
mod console_screen;
mod macros;
mod cpu_decoder;
mod cpu_instructions;
#[cfg(test)]
mod test_utils;
mod keyboard;
mod wasm_canvas_screen;
#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use wasm_mutex::Mutex;
use cpu::CPU;
use crate::console_screen::ConsoleScreen;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let program = read_rom();
    let mut cpu = CPU::new(Box::new(ConsoleScreen::new()));
    cpu.load_program(program);
    CPU::run(Arc::new(Mutex::new(cpu))).await;
    Ok(())
}

fn read_rom() -> Vec<u8> {
    let mut file = File::open("BLINKY").unwrap();
    let mut r: Vec<u8> = Vec::new();
    file.read_to_end(&mut r).unwrap();
    return r;
}