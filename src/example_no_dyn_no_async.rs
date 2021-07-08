// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=dd6719fffb225f86dcd965fd65a0c9fe

extern crate tokio_util; // 0.6.7;
extern crate tokio; // 1.7.1
extern crate futures; // 0.3.15

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

    pub fn run(&mut self) {
        let screen_draw = self.screen.request_animation_frame();
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
    fn request_animation_frame(&mut self) -> &mut ScreenDraw {
        let draw = &mut self.draw;
        return draw;
    }
}

pub struct ScreenDraw {}


