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
use wasm_bindgen::prelude::*;

use std::fs::File;
use std::io::Read;
use wasm_bindgen::JsCast;
use cpu::CPU;
use crate::console_screen::ConsoleScreen;
use crate::wasm_canvas_screen::WasmCanvasScreen;

#[wasm_bindgen]
pub struct ProgramHandle {
    _closure: Closure<dyn FnOnce()>,
}


#[wasm_bindgen]
pub fn init(program: &[u8], canvas: JsValue) -> Result<JsValue, JsValue> {
    match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(canvas) => {
            let mut cpu = CPU::new(Box::new(WasmCanvasScreen::new(canvas)));
            cpu.load_program(program.to_vec());
            cpu.run();
            Ok(Closure::once_into_js(move || {
                cpu.stop();
            }))
        }
        Err(_) => Err(JsValue::from_str("canvas argument not a HtmlCanvas")),
    }

}
