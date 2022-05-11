

use futures::future::LocalBoxFuture;




use crate::cpu_instructions::{X, Y};

pub struct IsCollision(pub bool);

pub fn toggle_pixel(state: &mut ScreenState, x: X, y: Y) -> IsCollision {
    let is_collision = state[y.0][x.0];
    state[y.0][x.0] = !is_collision;
    IsCollision(is_collision)
}

pub trait ScreenDraw {
    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision;
    fn repaint(&mut self);
    fn clear(&mut self);
    fn get_width(&self) -> usize {
        SCREEN_WIDTH
    }
    fn get_height(&self) -> usize {
        SCREEN_HEIGHT
    }
}

pub trait Screen: ScreenDraw {
    fn request_animation_frame(&self) -> LocalBoxFuture<()>;
}

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub type ScreenState = [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT];

pub fn make_zero_screen_state() -> ScreenState {
    [[false; SCREEN_WIDTH]; SCREEN_HEIGHT]
}