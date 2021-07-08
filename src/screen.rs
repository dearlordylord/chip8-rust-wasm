use std::future::Future;
use futures::{future::BoxFuture, FutureExt};
use std::pin::Pin;
use ux::u4;

pub trait ScreenDraw: std::marker::Send {
    fn toggle_pixel(&mut self, x: u4, y: u4);
    fn repaint(&mut self);
    fn clear(&mut self);
}

pub trait Screen {
    fn request_animation_frame(&mut self) -> BoxFuture<&mut dyn ScreenDraw>;
}