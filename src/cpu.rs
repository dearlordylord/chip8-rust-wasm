use crate::screen::{Screen, ScreenDraw};
use std::{println};
use ux::{u4, u12};
use std::ops::{Sub, Add, Deref, DerefMut};
use std::convert::{TryFrom, TryInto};
use tokio::time::{delay_for, Duration, Delay};
use std::borrow::BorrowMut;
const MEM_SIZE: usize = 4096;
const PROGRAM_START_ADDR: u16 = 0x0200;
const STACK_SIZE: usize = 16;
const REGISTERS_SIZE: usize = 16;

const STEPS_PER_CYCLE: usize = 10;
const SPEED: u64 = 60; // herz

const FONTS_LENGTH: usize = 80;
const FONTS: [u8; FONTS_LENGTH] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

type Mem = [u8; MEM_SIZE];

struct CPUState {
    mem: Mem,
    v: [u8; REGISTERS_SIZE],
    pc: u12,
    I: u12,
    stack: [u8; STACK_SIZE],
    sp: u4,
    repaint: bool,
    halted: bool,
    // timers
    dt: u32,
    st: u32,
}

impl CPUState {
    fn fetch(&self) -> u16 {
        return u16::from_be_bytes([self.mem[self.pci()], self.mem[self.pci() + 1]]);
    }
    fn pci(&self) -> usize {
        let r: u16 = self.pc.into();
        return r.into();
    }
    fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt = self.dt - 1;
        }
        if self.st > 0 {
            self.st = self.st - 1;
        }
    }
    // no more incs planned never; 2 fns is fine
    fn inc_pc_2(&mut self) {
        self.pc = self.pc.add(u12::new(2));
    }
    fn inc_pc_4(&mut self) {
        self.pc = self.pc.add(u12::new(4));
    }
}

pub struct CPU {
    state: CPUState,
    delay_ref: Option<Delay>,
    screen: Box<dyn Screen>,
}

fn load_font_set(mem: &mut Mem) {
    for i in 0..FONTS_LENGTH {
        mem[i] = FONTS[i];
    }
}

impl CPU {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut mem = [0; MEM_SIZE];
        load_font_set(&mut mem);
        Self {
            state: CPUState {
                mem,
                v: [0; REGISTERS_SIZE],
                pc: u12::new(PROGRAM_START_ADDR),
                I: u12::new(0),
                stack: [0; STACK_SIZE],
                sp: u4::new(0),
                repaint: false,
                halted: false,
                dt: 0,
                st: 0,
            },
            delay_ref: None,
            screen,
        }
    }
    pub fn load_program(&mut self, data: Vec<u8>) {
        assert!(u12::max_value().sub(u12::new(PROGRAM_START_ADDR)) >= u12::new(u16::try_from(data.len()).expect("Data len takes more than u16")));
        for (i, x) in data.iter().enumerate() {
            self.state.mem[usize::from(PROGRAM_START_ADDR) + i] = x.clone();
        }
    }

    pub async fn run(&mut self) {
        self.delay_ref = Some(delay_for(Duration::new(1 / SPEED, 0)));
        self.delay_ref.as_mut().unwrap().borrow_mut().await;
        if !self.state.halted {
            let screen_draw = self.screen.request_animation_frame().await;
            CPU::cycle(&mut self.state, screen_draw);
            self.run().await;
        }
    }

    pub fn stop(&mut self) {
        self.delay_ref = None;
    }

    fn cycle(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) {
        if state.halted {
            return;
        }
        for _ in 0..STEPS_PER_CYCLE {
            CPU::step(state, screen_draw);
        }
        state.update_timers();
        // if (this.st > 0) {
        //     this.audio.play();
        // } else {
        //     this.audio.stop();
        // }
    }

    fn step(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) {
        let opcode = state.fetch();
        let op = CPU::decode(opcode);
        CPU::execute(state, screen_draw, op);
        if state.repaint {
            screen_draw.repaint();
            state.repaint = false;
        }
    }

    fn execute(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw, op: impl Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()) {
        op(state, screen_draw);
    }

    fn decode(opcode: u16) -> Box<dyn Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()> {
        let x = X(((opcode & 0x0F00) >> 8).into());
        let y = Y(((opcode & 0x00F0) >> 4).into());
        let kk = KK(opcode & 0x00FF);
        let nnn = NNN(opcode & 0x0FFF);
        let n = N(opcode & 0x000F);
        println!("{}", opcode & 0xF000);
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => cls(),
                0x8000 => match opcode & 0x000F {
                    0x0000 => ld_vx_vy(x, y),
                    default => cls(),
                }
                default => cls()
            },
            // 0xA000 => { // ANNN: Sets I to the address NNN
            //     self.I = opcode & 0x0FFF;
            //     self.pc += 2;
            // },
            default => cls()
        }
    }

}

struct X(usize);
struct Y(usize);
struct KK(u16);
struct NNN(u16);
struct N(u16);
// type Instruction = dyn for<'a> Fn(&'a mut CPU);

/**
* <pre><code>0nnn - SYS addr</code></pre>
* Jump to a machine code routine at nnn.
* This instruction is only used on the old computers on which Chip-8 was originally implemented.
* It is ignored by modern interpreters.
*/
fn sys(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) {
    state.inc_pc_2();
}
/**
 * <pre><code>00E0 - CLS</code></pre>
 * Clears the display.
 */
fn cls() -> Box<dyn Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()> {
    return Box::new(|state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        screen_draw.clear();
        state.repaint = true;
        state.inc_pc_2();
    });
}
/**
 * <pre><code>8xy0 - LD Vx, Vy</code></pre>
 * Set Vx = Vy
 */
fn ld_vx_vy(x: X, y: Y) -> Box<dyn Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| state.v[x.0] = state.v[y.0]);
}



