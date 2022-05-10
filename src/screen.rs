use std::future::Future;
use futures::{future::BoxFuture};
use futures::future::LocalBoxFuture;
use futures::stream::Once;
use mockall::*;
use mockall::predicate::*;

use crate::cpu_instructions::{X, Y};

pub struct IsCollision(pub bool);

#[automock]
pub trait ScreenDraw {
    fn borrow_state(&mut self) -> &mut ScreenState;
    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision {
        let state = self.borrow_state();
        let is_collision = state[y.0][x.0];
        state[y.0][x.0] = !is_collision;
        IsCollision(is_collision)
    }
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