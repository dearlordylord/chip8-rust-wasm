use std::time::Duration;

use futures::{Future, future::BoxFuture, FutureExt};
use futures::future::LocalBoxFuture;
use tokio::time::{Delay, delay_for};

use crate::cpu_instructions::{X, Y};
use crate::screen::{IsCollision, make_zero_screen_state, Screen, SCREEN_HEIGHT, SCREEN_WIDTH, ScreenDraw, ScreenState};

pub struct ConsoleScreen {
    drawn: bool,
    state: ScreenState,
}

impl ScreenDraw for ConsoleScreen {
    fn borrow_state(&mut self) -> &mut ScreenState {
        &mut self.state
    }

    fn repaint(&mut self) {
        self.draw_console();
    }
    fn clear(&mut self) {
        self.state = make_zero_screen_state();
        self.flush_console();
    }

    fn get_width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn get_height(&self) -> usize {
        SCREEN_HEIGHT
    }
}

impl ConsoleScreen {
    pub fn new() -> Self {
        Self {
            drawn: false,
            state: make_zero_screen_state(),
        }
    }
    fn flush_console(&mut self) {
        if !self.drawn {return;}
        // https://stackoverflow.com/a/34837038/2123547
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        self.drawn = false;
    }
    fn draw_console(&mut self) {
        if self.drawn {
            self.flush_console();
        }
        print!("{}", self.state.iter().enumerate().map(|(y, _)| {
            self.state[0].iter().enumerate().map(|(x, _)| {
                if self.state[y][x] {
                    "*"
                } else {
                    " "
                }
            }).collect::<Vec<_>>().join("")
        }).collect::<Vec<_>>().join("\n"));
        self.drawn = true;
    }
}

impl Screen for ConsoleScreen {
    fn request_animation_frame(&self) -> LocalBoxFuture<()> {
        delay_for(Duration::new(0, 10000)).boxed_local()
    }
}