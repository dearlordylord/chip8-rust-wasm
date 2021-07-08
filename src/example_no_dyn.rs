// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=60457de45f802bc62a42fb3c75309663

extern crate tokio_util; // 0.6.7;
extern crate tokio; // 1.7.1
extern crate futures; // 0.3.15
use futures::{future::BoxFuture, FutureExt};
use std::{println};

fn main() -> std::io::Result<()> {
    let mut cpu = CPU::new(Box::new(Screen::new()));
    cpu.run();
    Ok(())
}

pub struct CPU {
    screen: Box<Screen>,
    // other fields I mutate during execution
}

impl CPU {
    pub fn new(screen: Box<Screen>) -> Self {
        Self {
            screen,
        }
    }

    pub async fn run(&mut self) {
        let screen_draw = self.screen.request_animation_frame().await;
        self.cycle(screen_draw);
    }


    fn cycle(&mut self, screen_draw: &mut ScreenDraw) {
        // mutate self fields; mutate screen_draw by calling its &mut self methods
    }
}

pub struct Screen {
    draw: ScreenDraw,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            draw: ScreenDraw {

            }
        }
    }
    fn request_animation_frame(&mut self) -> BoxFuture<&mut ScreenDraw> {
        let draw = &mut self.draw;
        let f = async {
            //... async work here
            draw
        };
        return f.boxed();
    }
}

pub struct ScreenDraw {}

impl ScreenDraw  {
    fn toggle_pixel(&mut self, x: u8, y: u8) {
        println!("toggle pixel {} {}", x, y);
    }
    fn repaint(&mut self) {
        println!("repaint");
    }
}


