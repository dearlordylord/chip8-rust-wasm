

use callback_future::CallbackFuture;

use futures::{FutureExt};
use futures::future::LocalBoxFuture;

use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::CanvasRenderingContext2d;

use crate::cpu_instructions::{X, Y};
use crate::screen::{IsCollision, Screen, ScreenDraw, ScreenState, make_zero_screen_state, toggle_pixel};
use wasm_bindgen::prelude::*;

pub struct WasmCanvasScreen {
    state: ScreenState,
    canvas: web_sys::CanvasRenderingContext2d,
}


impl ScreenDraw for WasmCanvasScreen {
    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision {
        let is_collision = toggle_pixel(&mut self.state, x, y.clone());
        self.draw_pixel(x, y, !is_collision.0);
        is_collision
    }
    fn repaint(&mut self) {
        self.state.iter().enumerate().for_each(|(y, r)| {
            r.iter().enumerate().for_each(|(x, c)| {
                self.draw_pixel(X(x), Y(y), *c);
            });
        });
    }
    fn clear(&mut self) {
        self.state = make_zero_screen_state();
        self.state.iter().enumerate().for_each(|(y, r)| {
            r.iter().enumerate().for_each(|(x, _)| {
                self.draw_pixel(X(x), Y(y), false);
            });
        });
    }
}

impl WasmCanvasScreen {
    pub fn new(canvas: web_sys::CanvasRenderingContext2d) -> Self {
        Self {
            state: make_zero_screen_state(),
            canvas,
        }
    }
    fn get_canvas_context(&self) -> &CanvasRenderingContext2d {
        &self.canvas
    }
    fn get_canvas_size(&self) -> (u32, u32) {
        (self.canvas.canvas().unwrap().width(), self.canvas.canvas().unwrap().height())
    }
    fn get_canvas_scale(&self) -> (f32, f32) {
        let (width, height) = self.get_canvas_size();
        let scale_x = width as f32 / self.get_width() as f32;
        let scale_y = height as f32 / self.get_height() as f32;
        (scale_x, scale_y)
    }
    fn draw_pixel(&self, x: X, y: Y, yes: bool) {
        let (scale_x, scale_y) = self.get_canvas_scale();
        let ctx = self.get_canvas_context();
        ctx.set_fill_style(&JsValue::from_str(if yes { "black" } else { "white" }));
        ctx.fill_rect((x.0 as f32 * scale_x).into(), (y.0 as f32 * scale_y).into(), scale_x.into(), scale_y.into());
    }
}

impl Screen for WasmCanvasScreen {
    fn request_animation_frame(&self) -> LocalBoxFuture<()> {
        let f = CallbackFuture::new(|complete| {
            window()
                .expect("Should have window")
                .request_animation_frame(Closure::once_into_js(move || {complete(())}).as_ref().unchecked_ref())
                .expect("should register `requestAnimationFrame` OK");
        });

        f.boxed_local()
    }
}