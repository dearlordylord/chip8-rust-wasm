use std::time::Duration;

use futures::{future::BoxFuture, FutureExt};
use tokio::time::delay_for;

use crate::cpu_instructions::{X, Y};
use crate::screen::{IsCollision, Screen, SCREEN_HEIGHT, SCREEN_WIDTH, ScreenDraw};

pub struct ConsoleScreen {
    draw: ConsoleScreenDraw,
}

type ScreenState = [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT];

pub struct ConsoleScreenDraw {
    drawn: bool,
    state: ScreenState,
}

pub fn make_zero_state() -> ScreenState {
    [[false; SCREEN_WIDTH]; SCREEN_HEIGHT]
}

impl ConsoleScreenDraw {
    fn new() -> Self {
        ConsoleScreenDraw {
            drawn: false,
            state: make_zero_state(),
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

impl ScreenDraw for ConsoleScreenDraw {

    fn toggle_pixel(&mut self, x: X, y: Y) -> IsCollision {
        let is_collision = self.state[y.0][x.0];
        self.state[y.0][x.0] = !is_collision;
        IsCollision(is_collision)
    }
    fn repaint(&mut self) {
        self.draw_console();
    }
    fn clear(&mut self) {
        self.state = make_zero_state();
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
            draw: ConsoleScreenDraw::new(),
        }
    }
}

impl Screen for ConsoleScreen {
    fn request_animation_frame(&mut self) -> BoxFuture<&mut dyn ScreenDraw> {
        let draw = &mut self.draw as &mut dyn ScreenDraw;
        let f = async move {
            delay_for(Duration::new(0, 10000)).await;
            draw
        };
        return f.boxed();
    }
}