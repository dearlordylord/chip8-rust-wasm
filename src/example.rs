// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8a39af92359c772a1889b11cbce414cd

extern crate tokio_util; // 0.6.7;
extern crate tokio; // 1.7.1
extern crate futures; // 0.3.15
use futures::{future::BoxFuture, FutureExt};
use std::{println};

fn main() -> std::io::Result<()> {
    let mut cpu = CPU::new(Box::new(ConsoleScreen::new()));
    cpu.run();
    Ok(())
}

pub struct CPU {
    screen: Box<dyn Screen>,
    // other fields I mutate during execution
}

impl CPU {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        Self {
            screen,
        }
    }

    pub async fn run(&mut self) {
        let screen_draw = self.screen.request_animation_frame().await;
        self.cycle(screen_draw);
    }


    fn cycle(&mut self, screen_draw: &mut dyn ScreenDraw) {
        // mutate self fields; mutate screen_draw by calling its &mut self methods
    }
}

pub trait ScreenDraw: std::marker::Send {
    fn toggle_pixel(&mut self, x: u8, y: u8);
    fn repaint(&mut self);
}

pub trait Screen {
    fn request_animation_frame(&mut self) -> BoxFuture<&mut dyn ScreenDraw>;
}

pub struct ConsoleScreen {
    draw: ConsoleScreenDraw,
}

pub struct ConsoleScreenDraw {

}

impl ScreenDraw for ConsoleScreenDraw {
    fn toggle_pixel(&mut self, x: u8, y: u8) {
        println!("toggle pixel {} {}", x, y);
    }
    fn repaint(&mut self) {
        println!("repaint");
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
        let f = async {
            //... async work here
            draw
        };
        return f.boxed();
    }
}


