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

use std::sync::Arc;
use std::time::Duration;
use fluvio_wasm_timer::Delay;
use wasm_bindgen::prelude::*;
#[macro_use]
extern crate lazy_static;





use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_mutex::Mutex;


use cpu::CPU;

use crate::wasm_canvas_screen::WasmCanvasScreen;

#[wasm_bindgen]
pub struct WasmProgram {
    // #[wasm_bindgen(skip)]
    cpu: Arc<Mutex<CPU>>,
}

// TODO DRY locks (marcos?)
#[wasm_bindgen]
impl WasmProgram {
    pub fn run(&self) {
        let clone = self.cpu.clone();
        spawn_local(async move {
            CPU::run(clone).await;
        })
    }
    pub fn stop(&mut self) {
        let clone = self.cpu.clone();
        spawn_local(async move {
            let mut guard = clone.lock().await;
            guard.stop();
        })
    }

    pub fn key_down(&mut self, k: usize) {
        let clone = self.cpu.clone();
        spawn_local(async move {
            let mut guard = clone.lock().await;
            guard.key_down(k);
        })
    }

    pub fn key_up(&mut self, k: usize) {
        let clone = self.cpu.clone();
        spawn_local(async move {
            let mut guard = clone.lock().await;
            guard.key_up(k);
        })
    }
}

#[wasm_bindgen]
pub fn init_program(program: &[u8], canvas: JsValue) -> Result<WasmProgram, JsValue> {
    match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(canvas) => {
            let mut cpu = CPU::new(Box::new(WasmCanvasScreen::new(canvas)));
            cpu.load_program(program.to_vec());
            Ok(WasmProgram { cpu: Arc::new(Mutex::new(cpu)) })
        }
        Err(_) => Err(JsValue::from_str("canvas argument not a HtmlCanvas")),
    }
}
