use std::{println};
use std::borrow::BorrowMut;
use std::convert::{TryFrom};
use std::ops::{Add, Sub};
use anyhow::Result;

use rand::prelude::ThreadRng;
use rand::Rng;
use tokio::time::{Delay, delay_for, Duration};
use ux::{u12, u4};

use crate::cpu_decoder::{decode};
use crate::macros::newtype_copy;
use crate::screen::{Screen, ScreenDraw};
use crate::keyboard::{KeyboardState};

const MEM_SIZE: usize = 4096;
const PROGRAM_START_ADDR: u16 = 0x0200;
const STACK_SIZE: usize = 16;
const REGISTERS_SIZE: usize = 16;

const STEPS_PER_CYCLE: usize = 10;
const SPEED: u64 = 60; // herz

const FONTS_LENGTH: usize = 80;
const FONTS: [MemPrimitive; FONTS_LENGTH] = [
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
type MemPrimitive = u8;
#[derive(Clone, Debug, Copy)]
pub(crate) struct MemValue(pub(crate) MemPrimitive);
// newtype_copy!(MemValue);

pub(crate) type Mem = [MemValue; MEM_SIZE];
#[derive(Clone, Debug)]
pub(crate) struct PC(pub(crate) u12);
#[derive(Clone, Debug)]
pub(crate) struct SP(pub(crate) u4);
#[derive(Clone, Debug)]
pub(crate) struct I(pub(crate) u12);
#[derive(Debug)]
pub(crate) struct V(pub(crate) MemPrimitive);
newtype_copy!(V);
#[derive(Clone, Debug)]
pub(crate) struct DT(pub(crate) MemPrimitive);
#[derive(Clone, Debug)]
pub(crate) struct ST(pub(crate) MemPrimitive);
#[derive(Clone, Debug)]
pub(crate) struct Repaint(pub(crate) bool);
#[derive(Clone, Debug)]
pub(crate) struct Halted(pub(crate) bool);

#[derive(Clone, Debug)]
pub struct CPUState {
    pub(crate) mem: Mem,
    /*
    16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal, ie. 0 through 15 in decimal, called V0 through `VF
    VF is also used as a flag register; many instructions will set it to either 1 or 0 based on some rule, for example using it as a carry flag
     */
    pub(crate) v: [V; REGISTERS_SIZE],
    pub(crate) pc: PC,
    /*
    Both the index register, program counter and stack entries are actually 16 bits long.
     In theory, they could increment beyond 4 kB of memory addresses.
     In practice, no CHIP-8 games do that.
     The early computers running CHIP-8 usually had less than 4 kB of RAM anyway.
     */
    pub(crate) i: I,
    pub(crate) stack: [u12; STACK_SIZE],
    pub(crate) sp: SP,
    pub(crate) repaint: Repaint,
    pub(crate) halted: Halted,
    // timers
    pub(crate) dt: DT,
    pub(crate) st: ST,
    pub(crate) quirks: CPUQuirks,
    // not in spec
    pub(crate) rng: ThreadRng,
    pub(crate) keyboard: KeyboardState,
}

/**
* Enables/disabled CPU quirks
* @property {boolean} shift - If enabled, VX is shifted and VY remains unchanged (default: false)
* @property {boolean} loadStore - If enabled, I is not incremented during load/store (default: false)
*/
#[derive(Clone, Debug)]
pub(crate) struct CPUQuirks {
    pub(crate) shift: bool,
    pub(crate) load_store: bool,
}

impl CPUQuirks {
    pub fn new() -> Self {
        CPUQuirks { shift: false, load_store: false }
    }
}

impl CPUState {
    fn fetch(&self) -> u16 {
        return u16::from_be_bytes([self.mem[self.pci()].0, self.mem[self.pci() + 1].0]);
    }
    pub(crate) fn pci(&self) -> usize {
        let r: u16 = self.pc.0.into();
        return r.into();
    }
    pub(crate) fn update_timers(&mut self) {
        if self.dt.0 > 0 {
            self.dt.0 = self.dt.0 - 1;
        }
        if self.st.0 > 0 {
            self.st.0 = self.st.0 - 1;
        }
    }
    pub(crate) fn run_rng(&mut self) -> u8 { // 0..255
        self.rng.gen()
    }
    // no more incs planned never; 2 fns is fine
    pub(crate) fn inc_pc_2(&mut self) {
        self.pc.0 = self.pc.0.add(u12::new(2));
    }
    // fn inc_pc_4(&mut self) {
    //     self.pc = self.pc.add(u12::new(4));
    // }
}

pub struct CPU<'a> {
    pub(crate) state: CPUState,
    delay_ref: Option<Delay>,
    screen: Box<dyn Screen + 'a>,
}

fn load_font_set(mem: &mut Mem) {
    for i in 0..FONTS_LENGTH {
        mem[i].0 = FONTS[i];
    }
}

impl<'a> CPU<'a> {
    pub fn new(screen: Box<dyn Screen + 'a>) -> Self {
        let mut mem = [MemValue(0); MEM_SIZE];
        load_font_set(&mut mem);
        Self {
            state: CPUState {
                mem,
                v: [V(0); REGISTERS_SIZE],
                pc: PC(u12::new(PROGRAM_START_ADDR)),
                i: I(u12::new(0)),
                stack: [u12::new(0); STACK_SIZE],
                sp: SP(u4::new(0)),
                repaint: Repaint(false),
                halted: Halted(false),
                dt: DT(0),
                st: ST(0),
                quirks: CPUQuirks::new(),
                rng: rand::thread_rng(),
                keyboard: KeyboardState::new(),
            },
            delay_ref: None,
            screen,
        }
    }
    pub fn load_program(&mut self, data: Vec<u8>) {
        assert!(u12::max_value().sub(u12::new(PROGRAM_START_ADDR)) >= u12::new(u16::try_from(data.len()).expect("Data len takes more than u16")));
        for (i, x) in data.iter().enumerate() {
            self.state.mem[usize::from(PROGRAM_START_ADDR) + i].0 = x.clone();
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.delay_ref = Some(delay_for(Duration::new(1 / SPEED, 0)));
            self.delay_ref.as_mut().unwrap().borrow_mut().await;
            if !self.state.halted.0 {
                let screen_draw = self.screen.request_animation_frame().await;
                match CPU::cycle(&mut self.state, screen_draw) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Error during cycle, {}. STOPPING", e);
                        self.stop();
                    }
                }
            }
        }

    }

    pub fn stop(&mut self) {
        self.delay_ref = None;
    }

    fn cycle(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) -> Result<()> {
        if state.halted.0 {
            return Ok(());
        }
        for _ in 0..STEPS_PER_CYCLE {
            CPU::step(state, screen_draw)?;
        }
        state.update_timers();
        Ok(())
        // if (this.st > 0) {
        //     this.audio.play();
        // } else {
        //     this.audio.stop();
        // }
    }

    pub(crate) fn step(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) -> StepResult {
        let opcode = state.fetch();
        let op = decode(opcode)?;
        // TODO result type, error type
        CPU::execute(state, screen_draw, op);
        if state.repaint.0 {
            screen_draw.repaint();
            state.repaint.0 = false;
        }
        StepResult::Ok(())
    }

    fn execute(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw, op: impl Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()) {
        op(state, screen_draw);
    }

}

type StepResult = Result<()>;