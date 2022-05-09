use futures::{future::BoxFuture};
use mockall::*;
use mockall::predicate::*;

use crate::cpu_instructions::{X, Y};

pub struct IsCollision(pub bool);

#[automock]
pub trait ScreenDraw: std::marker::Send {
    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision;
    fn repaint(&mut self);
    fn clear(&mut self);
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
}

pub trait Screen {
    fn request_animation_frame<'a>(&'a mut self) -> BoxFuture<'a, &'a mut dyn ScreenDraw>;
}

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;