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
#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use futures::TryFutureExt;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_mutex::Mutex;
use cpu::CPU;
use crate::console_screen::ConsoleScreen;
use crate::wasm_canvas_screen::WasmCanvasScreen;


#[wasm_bindgen]
pub struct ProgramHandle {
    _closure: Closure<dyn FnOnce()>,
}

/*
60, 33, 68, 79, 67, 84, 89, 80, 69, 32, 104, 116, 109, 108, 62, 10, 60, 104, 116, 109, 108, 32, 108, 97, 110, 103, 61, 34, 101, 110, 34, 62, 10, 32, 32, 60, 104, 101, 97, 100, 62, 10, 32, 32, 32, 32, 60, 109, 101, 116, 97, 32, 99, 104, 97, 114, 115, 101, 116, 61, 34, 117, 116, 102, 45, 56, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 108, 105, 110, 107, 32, 114, 101, 108, 61, 34, 105, 99, 111, 110, 34, 32, 104, 114, 101, 102, 61, 34, 47, 102, 97, 118, 105, 99, 111, 110, 46, 105, 99, 111, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 109, 101, 116, 97, 32, 110, 97, 109, 101, 61, 34, 118, 105, 101, 119, 112, 111, 114, 116, 34, 32, 99, 111, 110, 116, 101, 110, 116, 61, 34, 119, 105, 100, 116, 104, 61, 100, 101, 118, 105, 99, 101, 45, 119, 105, 100, 116, 104, 44, 32, 105, 110, 105, 116, 105, 97, 108, 45, 115, 99, 97, 108, 101, 61, 49, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 109, 101, 116, 97, 32, 110, 97, 109, 101, 61, 34, 116, 104, 101, 109, 101, 45, 99, 111, 108, 111, 114, 34, 32, 99, 111, 110, 116, 101, 110, 116, 61, 34, 35, 48, 48, 48, 48, 48, 48, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 109, 101, 116, 97, 10, 32, 32, 32, 32, 32, 32, 110, 97, 109, 101, 61, 34, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 34, 10, 32, 32, 32, 32, 32, 32, 99, 111, 110, 116, 101, 110, 116, 61, 34, 87, 101, 98, 32, 115, 105, 116, 101, 32, 99, 114, 101, 97, 116, 101, 100, 32, 117, 115, 105, 110, 103, 32, 99, 114, 101, 97, 116, 101, 45, 114, 101, 97, 99, 116, 45, 97, 112, 112, 34, 10, 32, 32, 32, 32, 47, 62, 10, 32, 32, 32, 32, 60, 108, 105, 110, 107, 32, 114, 101, 108, 61, 34, 97, 112, 112, 108, 101, 45, 116, 111, 117, 99, 104, 45, 105, 99, 111, 110, 34, 32, 104, 114, 101, 102, 61, 34, 47, 108, 111, 103, 111, 49, 57, 50, 46, 112, 110, 103, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 33, 45, 45, 10, 32, 32, 32, 32, 32, 32, 109, 97, 110, 105, 102, 101, 115, 116, 46, 106, 115, 111, 110, 32, 112, 114, 111, 118, 105, 100, 101, 115, 32, 109, 101, 116, 97, 100, 97, 116, 97, 32, 117, 115, 101, 100, 32, 119, 104, 101, 110, 32, 121, 111, 117, 114, 32, 119, 101, 98, 32, 97, 112, 112, 32, 105, 115, 32, 105, 110, 115, 116, 97, 108, 108, 101, 100, 32, 111, 110, 32, 97, 10, 32, 32, 32, 32, 32, 32, 117, 115, 101, 114, 39, 115, 32, 109, 111, 98, 105, 108, 101, 32, 100, 101, 118, 105, 99, 101, 32, 111, 114, 32, 100, 101, 115, 107, 116, 111, 112, 46, 32, 83, 101, 101, 32, 104, 116, 116, 112, 115, 58, 47, 47, 100, 101, 118, 101, 108, 111, 112, 101, 114, 115, 46, 103, 111, 111, 103, 108, 101, 46, 99, 111, 109, 47, 119, 101, 98, 47, 102, 117, 110, 100, 97, 109, 101, 110, 116, 97, 108, 115, 47, 119, 101, 98, 45, 97, 112, 112, 45, 109, 97, 110, 105, 102, 101, 115, 116, 47, 10, 32, 32, 32, 32, 45, 45, 62, 10, 32, 32, 32, 32, 60, 108, 105, 110, 107, 32, 114, 101, 108, 61, 34, 109, 97, 110, 105, 102, 101, 115, 116, 34, 32, 104, 114, 101, 102, 61, 34, 47, 109, 97, 110, 105, 102, 101, 115, 116, 46, 106, 115, 111, 110, 34, 32, 47, 62, 10, 32, 32, 32, 32, 60, 33, 45, 45, 10, 32, 32, 32, 32, 32, 32, 78, 111, 116, 105, 99, 101, 32, 116, 104, 101, 32, 117, 115, 101, 32, 111, 102, 32, 32, 105, 110, 32, 116, 104, 101, 32, 116, 97, 103, 115, 32, 97, 98, 111, 118, 101, 46, 10, 32, 32, 32, 32, 32, 32, 73, 116, 32, 119, 105, 108, 108, 32, 98, 101, 32, 114, 101, 112, 108, 97, 99, 101, 100, 32, 119, 105, 116, 104, 32, 116, 104, 101, 32, 85, 82, 76, 32, 111, 102, 32, 116, 104, 101, 32, 96, 112, 117, 98, 108, 105, 99, 96, 32, 102, 111, 108, 100, 101, 114, 32, 100, 117, 114, 105, 110, 103, 32, 116, 104, 101, 32, 98, 117, 105, 108, 100, 46, 10, 32, 32, 32, 32, 32, 32, 79, 110, 108, 121, 32, 102, 105, 108, 101, 115, 32, 105, 110, 115, 105, 100, 101, 32, 116, 104, 101, 32, 96, 112, 117, 98, 108, 105, 99, 96, 32, 102, 111, 108, 100, 101, 114, 32, 99, 97, 110, 32, 98, 101, 32, 114, 101, 102, 101, 114, 101, 110, 99, 101, 100, 32, 102, 114, 111, 109, 32, 116, 104, 101, 32, 72, 84, 77, 76, 46, 10, 10, 32, 32, 32, 32, 32, 32, 85, 110, 108, 105, 107, 101, 32, 34, 47, 102, 97, 118, 105, 99, 111, 110, 46, 105, 99, 111, 34, 32, 111, 114, 32, 34, 102, 97, 118, 105, 99, 111, 110, 46, 105, 99, 111, 34, 44, 32, 34, 47, 102, 97, 118, 105, 99, 111, 110, 46, 105, 99, 111, 34, 32, 119, 105, 108, 108, 10, 32, 32, 32, 32, 32, 32, 119, 111, 114, 107, 32, 99, 111, 114, 114, 101, 99, 116, 108, 121, 32, 98, 111, 116, 104, 32, 119, 105, 116, 104, 32, 99, 108, 105, 101, 110, 116, 45, 115, 105, 100, 101, 32, 114, 111, 117, 116, 105, 110, 103, 32, 97, 110, 100, 32, 97, 32, 110, 111, 110, 45, 114, 111, 111, 116, 32, 112, 117, 98, 108, 105, 99, 32, 85, 82, 76, 46, 10, 32, 32, 32, 32, 32, 32, 76, 101, 97, 114, 110, 32, 104, 111, 119, 32, 116, 111, 32, 99, 111, 110, 102, 105, 103, 117, 114, 101, 32, 97, 32, 110, 111, 110, 45, 114, 111, 111, 116, 32, 112, 117, 98, 108, 105, 99, 32, 85, 82, 76, 32, 98, 121, 32, 114, 117, 110, 110, 105, 110, 103, 32, 96, 110, 112, 109, 32, 114, 117, 110, 32, 98, 117, 105, 108, 100, 96, 46, 10, 32, 32, 32, 32, 45, 45, 62, 10, 32, 32, 32, 32, 60, 116, 105, 116, 108, 101, 62, 82, 101, 97, 99, 116, 32, 65, 112, 112, 60, 47, 116, 105, 116, 108, 101, 62, 10, 32, 32, 60, 115, 99, 114, 105, 112, 116, 32, 100, 101, 102, 101, 114, 32, 115, 114, 99, 61, 34, 47, 115, 116, 97, 116, 105, 99, 47, 106, 115, 47, 98, 117, 110, 100, 108, 101, 46, 106, 115, 34, 62, 60, 47, 115, 99, 114, 105, 112, 116, 62, 60, 47, 104, 101, 97, 100, 62, 10, 32, 32, 60, 98, 111, 100, 121, 62, 10, 32, 32, 32, 32, 60, 110, 111, 115, 99, 114, 105, 112, 116, 62, 89, 111, 117, 32, 110, 101, 101, 100, 32, 116, 111, 32, 101, 110, 97, 98, 108, 101, 32, 74, 97, 118, 97, 83, 99, 114, 105, 112, 116, 32, 116, 111, 32, 114, 117, 110, 32, 116, 104, 105, 115, 32, 97, 112, 112, 46, 60, 47, 110, 111, 115, 99, 114, 105, 112, 116, 62, 10, 32, 32, 32, 32, 60, 100, 105, 118, 32, 105, 100, 61, 34, 114, 111, 111, 116, 34, 62, 60, 47, 100, 105, 118, 62, 10, 32, 32, 32, 32, 60, 33, 45, 45, 10, 32, 32, 32, 32, 32, 32, 84, 104, 105, 115, 32, 72, 84, 77, 76, 32, 102, 105, 108, 101, 32, 105, 115, 32, 97, 32, 116, 101, 109, 112, 108, 97, 116, 101, 46, 10, 32, 32, 32, 32, 32, 32, 73, 102, 32, 121, 111, 117, 32, 111, 112, 101, 110, 32, 105, 116, 32, 100, 105, 114, 101, 99, 116, 108, 121, 32, 105, 110, 32, 116, 104, 101, 32, 98, 114, 111, 119, 115, 101, 114, 44, 32, 121, 111, 117, 32, 119, 105, 108, 108, 32, 115, 101, 101, 32, 97, 110, 32, 101, 109, 112, 116, 121, 32, 112, 97, 103, 101, 46, 10, 10, 32, 32, 32, 32, 32, 32, 89, 111, 117, 32, 99, 97, 110, 32, 97, 100, 100, 32, 119, 101, 98, 102, 111, 110, 116, 115, 44, 32, 109, 101, 116, 97, 32, 116, 97, 103, 115, 44, 32, 111, 114, 32, 97, 110, 97, 108, 121, 116, 105, 99, 115, 32, 116, 111, 32, 116, 104, 105, 115, 32, 102, 105, 108, 101, 46, 10, 32, 32, 32, 32, 32, 32, 84, 104, 101, 32, 98, 117, 105, 108, 100, 32, 115, 116, 101, 112, 32, 119, 105, 108, 108, 32, 112, 108, 97, 99, 101, 32, 116, 104, 101, 32, 98, 117, 110, 100, 108, 101, 100, 32, 115, 99, 114, 105, 112, 116, 115, 32, 105, 110, 116, 111, 32, 116, 104, 101, 32, 60, 98, 111, 100, 121, 62, 32, 116, 97, 103, 46, 10, 10, 32, 32, 32, 32, 32, 32, 84, 111, 32, 98, 101, 103, 105, 110, 32, 116, 104, 101, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 44, 32, 114, 117, 110, 32, 96, 110, 112, 109, 32, 115, 116, 97, 114, 116, 96, 32, 111, 114, 32, 96, 121, 97, 114, 110, 32, 115, 116, 97, 114, 116, 96, 46, 10, 32, 32, 32, 32, 32, 32, 84, 111, 32, 99, 114, 101, 97, 116, 101, 32, 97, 32, 112, 114, 111, 100, 117, 99, 116, 105, 111, 110, 32, 98, 117, 110, 100, 108, 101, 44, 32, 117, 115, 101, 32, 96, 110, 112, 109, 32, 114, 117, 110, 32, 98, 117, 105, 108, 100, 96, 32, 111, 114, 32, 96, 121, 97, 114, 110, 32, 98, 117, 105, 108, 100, 96, 46, 10, 32, 32, 32, 32, 45, 45, 62, 10, 32, 32, 60, 47, 98, 111, 100, 121, 62, 10, 60, 47, 104, 116, 109, 108, 62, 10
18, 26, 50, 46, 48, 48, 32, 67, 46, 32, 69, 103, 101, 98, 101, 114, 103, 32, 49, 56, 47, 56, 45, 39, 57, 49, 128, 3, 129, 19, 168, 200, 241, 85, 96, 5, 168, 204, 240, 85, 135, 115, 134, 99, 39, 114, 0, 224, 39, 148, 110, 64, 135, 226, 110, 39, 135, 225, 104, 26, 105, 12, 106, 56, 107, 0, 108, 2, 109, 26, 39, 80, 168, 237, 218, 180, 220, 212, 35, 208, 62, 0, 18, 124, 168, 204, 240, 101, 133, 0, 196, 255, 132, 82, 36, 246, 196, 255, 132, 82, 38, 30, 96, 1, 224, 161, 39, 214, 54, 247, 18, 78, 142, 96, 40, 122, 110, 100, 40, 122, 39, 214, 18, 42, 240, 7, 64, 0, 19, 16, 128, 128, 128, 6, 129, 160, 129, 6, 128, 21, 64, 0, 18, 154, 64, 1, 18, 154, 64, 255, 18, 154, 18, 200, 128, 144, 128, 6, 129, 176, 129, 6, 128, 21, 64, 0, 18, 178, 64, 1, 18, 178, 64, 255, 18, 178, 18, 200, 168, 237, 218, 180, 106, 56, 107, 0, 218, 180, 110, 243, 135, 226, 110, 4, 135, 225, 110, 50, 40, 122, 128, 128, 128, 6, 129, 192, 129, 6, 128, 21, 64, 0, 18, 224, 64, 1, 18, 224, 64, 255, 18, 224, 18, 84, 128, 144, 128, 6, 129, 208, 129, 6, 128, 21, 64, 0, 18, 248, 64, 1, 18, 248, 64, 255, 18, 248, 18, 84, 168, 237, 220, 212, 108, 2, 109, 26, 220, 212, 110, 207, 135, 226, 110, 32, 135, 225, 110, 25, 40, 122, 18, 84, 96, 63, 40, 168, 39, 80, 168, 237, 218, 180, 220, 212, 110, 64, 135, 227, 128, 112, 128, 226, 48, 0, 18, 50, 142, 96, 40, 122, 40, 138, 0, 224, 102, 17, 103, 10, 168, 202, 39, 230, 102, 17, 103, 16, 168, 200, 39, 230, 100, 0, 101, 8, 102, 0, 103, 15, 171, 25, 212, 105, 171, 34, 213, 105, 96, 3, 40, 168, 62, 0, 19, 198, 171, 25, 212, 105, 171, 34, 213, 105, 116, 2, 117, 2, 52, 48, 19, 72, 171, 25, 212, 105, 171, 34, 213, 105, 96, 3, 40, 168, 62, 0, 19, 198, 171, 25, 212, 105, 171, 34, 213, 105, 118, 2, 54, 22, 19, 104, 171, 25, 212, 105, 171, 34, 213, 105, 96, 3, 40, 168, 62, 0, 19, 198, 171, 25, 212, 105, 171, 34, 213, 105, 116, 254, 117, 254, 52, 0, 19, 134, 171, 25, 212, 105, 171, 34, 213, 105, 96, 3, 40, 168, 62, 0, 19, 198, 171, 25, 212, 105, 171, 34, 213, 105, 118, 254, 54, 0, 19, 166, 19, 72, 171, 34, 213, 105, 171, 43, 213, 105, 18, 26, 131, 112, 110, 3, 131, 226, 132, 128, 133, 144, 110, 6, 238, 161, 20, 50, 110, 3, 238, 161, 20, 74, 110, 8, 238, 161, 20, 98, 110, 7, 238, 161, 20, 122, 67, 3, 117, 2, 67, 0, 117, 254, 67, 2, 116, 2, 67, 1, 116, 254, 128, 64, 129, 80, 39, 186, 130, 0, 110, 8, 128, 226, 48, 0, 20, 146, 110, 7, 128, 32, 130, 226, 66, 5, 20, 154, 66, 6, 20, 178, 66, 7, 20, 236, 39, 80, 110, 252, 135, 226, 135, 49, 136, 64, 137, 80, 23, 80, 128, 64, 129, 80, 113, 2, 39, 186, 130, 0, 110, 8, 128, 226, 48, 0, 19, 242, 99, 3, 117, 2, 20, 14, 128, 64, 129, 80, 113, 254, 39, 186, 130, 0, 110, 8, 128, 226, 48, 0, 19, 242, 99, 0, 117, 254, 20, 14, 128, 64, 129, 80, 112, 2, 39, 186, 130, 0, 110, 8, 128, 226, 48, 0, 19, 242, 99, 2, 116, 2, 20, 14, 128, 64, 129, 80, 112, 254, 39, 186, 130, 0, 110, 8, 128, 226, 48, 0, 19, 242, 99, 1, 116, 254, 20, 14, 39, 80, 216, 148, 142, 240, 0, 238, 110, 240, 128, 226, 128, 49, 240, 85, 168, 241, 212, 84, 118, 1, 97, 5, 240, 7, 64, 0, 241, 24, 20, 36, 110, 240, 128, 226, 128, 49, 240, 85, 168, 245, 212, 84, 118, 4, 128, 160, 129, 176, 39, 186, 110, 240, 128, 226, 48, 0, 20, 210, 110, 12, 135, 227, 128, 192, 129, 208, 39, 186, 110, 240, 128, 226, 48, 0, 20, 228, 110, 48, 135, 227, 96, 255, 240, 24, 240, 21, 20, 36, 67, 1, 100, 58, 67, 2, 100, 0, 20, 36, 130, 112, 131, 112, 110, 12, 130, 226, 128, 160, 129, 176, 39, 186, 168, 237, 110, 240, 128, 226, 48, 0, 21, 36, 218, 180, 66, 12, 123, 2, 66, 0, 123, 254, 66, 8, 122, 2, 66, 4, 122, 254, 218, 180, 0, 238, 110, 128, 241, 7, 49, 0, 21, 212, 52, 0, 21, 212, 129, 0, 131, 14, 63, 0, 21, 86, 131, 144, 131, 181, 79, 0, 21, 140, 51, 0, 21, 116, 135, 227, 131, 128, 131, 165, 79, 0, 21, 188, 51, 0, 21, 164, 135, 227, 21, 212, 131, 128, 131, 165, 79, 0, 21, 188, 51, 0, 21, 164, 135, 227, 131, 144, 131, 181, 79, 0, 21, 140, 51, 0, 21, 116, 135, 227, 21, 212, 99, 64, 129, 50, 65, 0, 21, 212, 218, 180, 123, 2, 218, 180, 110, 243, 135, 226, 98, 12, 135, 33, 0, 238, 99, 16, 129, 50, 65, 0, 21, 212, 218, 180, 123, 254, 218, 180, 110, 243, 135, 226, 98, 0, 135, 33, 0, 238, 99, 32, 129, 50, 65, 0, 21, 212, 218, 180, 122, 2, 218, 180, 110, 243, 135, 226, 98, 8, 135, 33, 0, 238, 99, 128, 129, 50, 65, 0, 21, 212, 218, 180, 122, 254, 218, 180, 110, 243, 135, 226, 98, 4, 135, 33, 0, 238, 193, 240, 128, 18, 48, 0, 21, 228, 110, 12, 135, 227, 130, 227, 21, 14, 218, 180, 128, 14, 79, 0, 21, 242, 98, 4, 122, 254, 22, 20, 128, 14, 79, 0, 21, 254, 98, 12, 123, 2, 22, 20, 128, 14, 79, 0, 22, 10, 98, 8, 122, 2, 22, 20, 128, 14, 79, 0, 21, 220, 98, 0, 123, 254, 218, 180, 110, 243, 135, 226, 135, 33, 0, 238, 130, 112, 131, 112, 110, 48, 130, 226, 128, 192, 129, 208, 39, 186, 168, 237, 110, 240, 128, 226, 48, 0, 22, 76, 220, 212, 66, 48, 125, 2, 66, 0, 125, 254, 66, 32, 124, 2, 66, 16, 124, 254, 220, 212, 0, 238, 110, 128, 241, 7, 49, 0, 23, 4, 52, 0, 23, 4, 129, 0, 131, 14, 79, 0, 22, 126, 131, 144, 131, 213, 79, 0, 22, 182, 51, 0, 22, 156, 135, 227, 131, 128, 131, 197, 79, 0, 22, 234, 51, 0, 22, 208, 135, 227, 23, 4, 131, 128, 131, 197, 79, 0, 22, 234, 51, 0, 22, 208, 135, 227, 131, 144, 131, 213, 79, 0, 22, 182, 51, 0, 22, 156, 135, 227, 23, 4, 99, 64, 129, 50, 65, 0, 23, 4, 220, 212, 125, 2, 220, 212, 135, 227, 110, 207, 135, 226, 98, 48, 135, 33, 0, 238, 99, 16, 129, 50, 65, 0, 23, 4, 220, 212, 125, 254, 220, 212, 135, 227, 110, 207, 135, 226, 98, 0, 135, 33, 0, 238, 99, 32, 129, 50, 65, 0, 23, 4, 220, 212, 124, 2, 220, 212, 135, 227, 110, 207, 135, 226, 98, 32, 135, 33, 0, 238, 99, 128, 129, 50, 65, 0, 23, 4, 220, 212, 124, 254, 220, 212, 135, 227, 110, 207, 135, 226, 98, 16, 135, 33, 0, 238, 193, 240, 128, 18, 48, 0, 23, 22, 135, 227, 110, 48, 135, 227, 130, 227, 22, 54, 220, 212, 128, 14, 79, 0, 23, 36, 98, 144, 124, 254, 23, 70, 128, 14, 79, 0, 23, 48, 98, 48, 125, 2, 23, 70, 128, 14, 79, 0, 23, 60, 98, 160, 124, 2, 23, 70, 128, 14, 79, 0, 23, 12, 98, 0, 125, 254, 220, 212, 110, 79, 135, 226, 135, 33, 0, 238, 128, 112, 110, 3, 128, 226, 128, 14, 129, 128, 129, 148, 110, 2, 129, 226, 65, 0, 112, 1, 128, 14, 128, 14, 168, 205, 240, 30, 216, 148, 142, 240, 0, 238, 110, 0, 169, 25, 254, 30, 254, 30, 254, 30, 254, 30, 243, 101, 171, 52, 254, 30, 254, 30, 254, 30, 254, 30, 243, 85, 126, 1, 62, 128, 23, 116, 0, 238, 130, 35, 131, 51, 110, 15, 128, 32, 129, 48, 39, 190, 128, 226, 128, 14, 168, 249, 240, 30, 210, 50, 114, 2, 50, 64, 23, 154, 130, 35, 115, 2, 67, 32, 0, 238, 23, 154, 112, 2, 113, 2, 128, 6, 129, 6, 129, 14, 129, 14, 129, 14, 129, 14, 171, 52, 241, 30, 241, 30, 240, 30, 240, 101, 0, 238, 168, 204, 240, 101, 128, 6, 240, 85, 96, 1, 224, 161, 23, 224, 0, 238, 241, 101, 110, 1, 132, 67, 130, 0, 131, 16, 101, 16, 131, 85, 79, 0, 130, 229, 79, 0, 24, 12, 101, 39, 130, 85, 79, 0, 24, 12, 128, 32, 129, 48, 132, 228, 23, 240, 244, 41, 214, 117, 118, 6, 132, 67, 130, 0, 131, 16, 101, 232, 131, 85, 79, 0, 130, 229, 79, 0, 24, 52, 101, 3, 130, 85, 79, 0, 24, 52, 128, 32, 129, 48, 132, 228, 24, 24, 244, 41, 214, 117, 118, 6, 132, 67, 130, 0, 131, 16, 101, 100, 131, 85, 79, 0, 130, 229, 79, 0, 24, 84, 128, 32, 129, 48, 132, 228, 24, 64, 244, 41, 214, 117, 118, 6, 132, 67, 130, 0, 131, 16, 101, 10, 131, 85, 79, 0, 24, 110, 129, 48, 132, 228, 24, 96, 244, 41, 214, 117, 118, 6, 241, 41, 214, 117, 0, 238, 168, 200, 241, 101, 129, 228, 63, 0, 112, 1, 168, 200, 241, 85, 0, 238, 168, 200, 243, 101, 142, 0, 142, 37, 79, 0, 0, 238, 62, 0, 24, 162, 142, 16, 142, 53, 79, 0, 0, 238, 168, 202, 241, 85, 0, 238, 142, 227, 98, 15, 99, 255, 97, 16, 226, 161, 24, 196, 129, 52, 49, 0, 24, 176, 97, 16, 128, 52, 48, 0, 24, 176, 0, 238, 110, 1, 0, 238, 0, 0, 0, 0, 5, 0, 80, 112, 32, 0, 80, 112, 32, 0, 96, 48, 96, 0, 96, 48, 96, 0, 48, 96, 48, 0, 48, 96, 48, 0, 32, 112, 80, 0, 32, 112, 80, 0, 32, 112, 112, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 192, 0, 0, 0, 128, 128, 0, 0, 192, 128, 128, 128, 192, 0, 128, 0, 12, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 13, 12, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 13, 10, 101, 5, 5, 5, 5, 229, 5, 5, 229, 5, 5, 5, 5, 197, 10, 10, 101, 5, 5, 5, 5, 229, 5, 5, 229, 5, 5, 5, 5, 197, 10, 10, 5, 12, 8, 8, 15, 5, 12, 13, 5, 8, 8, 8, 13, 5, 14, 15, 5, 12, 8, 8, 15, 5, 12, 13, 5, 8, 8, 8, 13, 5, 10, 10, 5, 10, 101, 6, 5, 149, 10, 10, 53, 5, 5, 197, 10, 53, 5, 5, 149, 10, 101, 5, 5, 149, 10, 10, 53, 5, 6, 197, 10, 5, 10, 10, 5, 15, 5, 8, 8, 8, 8, 8, 12, 8, 15, 5, 8, 8, 8, 8, 8, 15, 5, 8, 8, 12, 8, 8, 8, 8, 15, 5, 15, 5, 10, 10, 117, 5, 181, 5, 5, 5, 5, 197, 10, 101, 5, 181, 5, 229, 5, 5, 229, 5, 181, 5, 197, 10, 101, 5, 5, 5, 5, 181, 5, 213, 10, 10, 5, 12, 8, 8, 8, 8, 13, 5, 15, 5, 12, 8, 15, 5, 8, 15, 5, 8, 8, 13, 5, 15, 5, 12, 8, 8, 8, 8, 13, 5, 10, 15, 5, 15, 101, 5, 5, 197, 10, 53, 229, 149, 10, 101, 5, 176, 5, 5, 181, 5, 197, 10, 53, 229, 149, 10, 101, 5, 5, 197, 15, 5, 15, 7, 116, 5, 213, 8, 15, 5, 14, 15, 5, 8, 15, 5, 12, 8, 8, 8, 8, 13, 5, 8, 15, 5, 8, 15, 5, 8, 15, 117, 5, 212, 7, 10, 5, 10, 53, 5, 5, 245, 5, 5, 181, 5, 5, 213, 8, 8, 13, 12, 8, 15, 117, 5, 5, 181, 5, 5, 245, 5, 5, 149, 10, 5, 10, 10, 5, 8, 8, 8, 13, 5, 12, 8, 8, 8, 13, 53, 5, 197, 10, 10, 101, 5, 149, 12, 8, 8, 8, 13, 5, 12, 8, 8, 15, 5, 10, 10, 117, 5, 6, 197, 10, 5, 8, 8, 8, 8, 8, 8, 15, 5, 8, 15, 5, 8, 8, 8, 8, 8, 8, 15, 5, 10, 101, 6, 5, 213, 10, 10, 5, 12, 13, 5, 10, 53, 5, 5, 5, 5, 229, 5, 5, 245, 5, 5, 245, 5, 5, 229, 5, 5, 5, 5, 149, 10, 5, 12, 13, 5, 10, 10, 5, 8, 15, 5, 8, 8, 8, 8, 8, 15, 5, 12, 13, 5, 8, 15, 5, 12, 13, 5, 8, 8, 8, 8, 8, 15, 5, 8, 15, 5, 10, 10, 53, 5, 5, 181, 5, 5, 5, 5, 5, 5, 149, 10, 10, 53, 5, 5, 149, 10, 10, 53, 5, 5, 5, 5, 5, 5, 181, 5, 5, 149, 10, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 15, 60, 66, 153, 153, 66, 60, 1, 16, 15, 120, 132, 50, 50, 132, 120, 0, 16, 224, 120, 252, 254, 254, 132, 120, 0, 16, 224
 */


// #[wasm_bindgen]
// pub fn init_program(program: &[u8], canvas: JsValue) -> Result<JsValue, JsValue> {
//     use web_sys::console;
//
//     match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
//         Ok(canvas) => {
//             let mut cpu = CPU::new(Box::new(WasmCanvasScreen::new(canvas)));
//             console::log_1(&program.to_vec().iter().fold(String::new(), |a, b| a + b.to_string().trim() + ", ").to_string().into());
//             cpu.load_program(program.to_vec());
//             let mut cpu_locked = Arc::new(Mutex::new(cpu));
//             let mut cpu_locked_for_deinit = cpu_locked.clone();
//             spawn_local(async move {
//                 loop {
//                     let arc_clone = cpu_locked.clone();
//                     let mut cpu = arc_clone.lock().await;
//                     if cpu.is_done() {
//                         break;
//                     }
//                     cpu.run().await;
//                 }
//             });
//             console::log_1(&"after run".into());
//             Ok(Closure::once_into_js( move || {
//                 cpu_locked_for_deinit.clone().lock().into().stop();
//             }))
//         }
//         Err(_) => Err(JsValue::from_str("canvas argument not a HtmlCanvas")),
//     }
//
// }

#[wasm_bindgen]
pub fn init_program(program: &[u8], canvas: JsValue) -> Result<CPU, JsValue> {
    match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(canvas) => {
            let mut cpu = CPU::new(Box::new(WasmCanvasScreen::new(canvas)));
            cpu.load_program(program.to_vec());
            return Ok(cpu);
        }
        Err(_) => Err(JsValue::from_str("canvas argument not a HtmlCanvas")),
    }
}
