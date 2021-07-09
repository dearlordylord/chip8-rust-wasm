use crate::screen::{Screen, ScreenDraw, IsCollision};
use crate::cpu::{X, Y};
use futures::{future::BoxFuture, FutureExt};
use std::future::Future;
use std::pin::Pin;
use tokio::time::delay_for;
use std::time::Duration;
use ux::u4;

pub struct ConsoleScreen {
    draw: ConsoleScreenDraw,
}

pub struct ConsoleScreenDraw {

}

impl ScreenDraw for ConsoleScreenDraw {
    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision {
        println!("toggle pixel {} {}", x.0, y.0);
        IsCollision(false)
    }
    fn repaint(&mut self) {
        println!("repaint");
    }
    fn clear(&mut self) {
        println!("clear");
    }
}

impl ConsoleScreen {
    pub fn new() -> Self {
        Self {
            draw: ConsoleScreenDraw {

            }
        }
    }
}

impl Screen for ConsoleScreen {
    fn request_animation_frame(&mut self) -> BoxFuture<&mut dyn ScreenDraw> {
        let draw = &mut self.draw as &mut dyn ScreenDraw;
        let f = async move {
            println!("request animation frame");
            delay_for(Duration::new(0, 10000)).await;
            println!("request animation frame done");
            draw
        };
        return f.boxed();
    }
}