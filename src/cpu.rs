use crate::screen::{Screen, ScreenDraw};
use crate::macros::newtype_copy;
use rand::Rng;
use std::{println};
use ux::{u4, u12};
use std::ops::{Sub, Add, Deref, DerefMut};
use std::convert::{TryFrom, TryInto};
use tokio::time::{delay_for, Duration, Delay};
use std::borrow::BorrowMut;
use rand::prelude::ThreadRng;

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
struct MemValue(MemPrimitive);
newtype_copy!(MemValue);

type Mem = [MemValue; MEM_SIZE];
struct PC(u12);
struct SP(u4);
struct I(u12);
struct V(MemPrimitive);
newtype_copy!(V);
struct DT(MemPrimitive);
struct ST(MemPrimitive);
struct Repaint(bool);
struct Halted(bool);

struct CPUState {
    mem: Mem,
    /*
    16 8-bit (one byte) general-purpose variable registers numbered 0 through F hexadecimal, ie. 0 through 15 in decimal, called V0 through `VF
    VF is also used as a flag register; many instructions will set it to either 1 or 0 based on some rule, for example using it as a carry flag
     */
    v: [V; REGISTERS_SIZE],
    pc: PC,
    /*
    Both the index register, program counter and stack entries are actually 16 bits long.
     In theory, they could increment beyond 4 kB of memory addresses.
     In practice, no CHIP-8 games do that.
     The early computers running CHIP-8 usually had less than 4 kB of RAM anyway.
     */
    I: I,
    stack: [u12; STACK_SIZE],
    sp: SP,
    repaint: Repaint,
    halted: Halted,
    // timers
    dt: DT,
    st: ST,
    quirks: CPUQuirks,
    // not in spec
    rng: ThreadRng,
}

/**
* Enables/disabled CPU quirks
* @property {boolean} shift - If enabled, VX is shifted and VY remains unchanged (default: false)
* @property {boolean} loadStore - If enabled, I is not incremented during load/store (default: false)
*/
struct CPUQuirks {
    shift: bool,
    load_store: bool,
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
    fn pci(&self) -> usize {
        let r: u16 = self.pc.0.into();
        return r.into();
    }
    fn update_timers(&mut self) {
        if self.dt.0 > 0 {
            self.dt.0 = self.dt.0 - 1;
        }
        if self.st.0 > 0 {
            self.st.0 = self.st.0 - 1;
        }
    }
    // no more incs planned never; 2 fns is fine
    fn inc_pc_2(&mut self) {
        self.pc.0 = self.pc.0.add(u12::new(2));
    }
    // fn inc_pc_4(&mut self) {
    //     self.pc = self.pc.add(u12::new(4));
    // }
}

pub struct CPU {
    state: CPUState,
    delay_ref: Option<Delay>,
    screen: Box<dyn Screen>,
}

fn load_font_set(mem: &mut Mem) {
    for i in 0..FONTS_LENGTH {
        mem[i].0 = FONTS[i];
    }
}

impl CPU {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut mem = [MemValue(0); MEM_SIZE];
        load_font_set(&mut mem);
        Self {
            state: CPUState {
                mem,
                v: [V(0); REGISTERS_SIZE],
                pc: PC(u12::new(PROGRAM_START_ADDR)),
                I: I(u12::new(0)),
                stack: [u12::new(0); STACK_SIZE],
                sp: SP(u4::new(0)),
                repaint: Repaint(false),
                halted: Halted(false),
                dt: DT(0),
                st: ST(0),
                quirks: CPUQuirks::new(),
                rng: rand::thread_rng(),
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
                CPU::cycle(&mut self.state, screen_draw);
            }
        }

    }

    pub fn stop(&mut self) {
        self.delay_ref = None;
    }

    fn cycle(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw) {
        if state.halted.0 {
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
        if state.repaint.0 {
            screen_draw.repaint();
            state.repaint.0 = false;
        }
    }

    fn execute(state: &mut CPUState, screen_draw: &mut dyn ScreenDraw, op: impl Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()) {
        op(state, screen_draw);
    }

    fn decode(opcode: u16) -> Box<dyn Fn(&mut CPUState, &mut dyn ScreenDraw) -> ()> {
        let x = X(((opcode & 0x0F00) >> 8).into());
        let y = Y(((opcode & 0x00F0) >> 4).into());
        let kk = KK((opcode & 0x00FF).to_be_bytes()[1]);
        let nnn = NNN(u12::new(opcode & 0x0FFF));
        let n = N(opcode & 0x000F);
        println!("{}", opcode & 0xF000);
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => cls(),
                0x8000 => match opcode & 0x000F {
                    0x0000 => ld_vx_vy(x, y),
                    0x0001 => or_vx_vy(x, y),
                    0x0002 => ld_vx_kk(x, kk),
                    0x0003 => ld_vx_kk(x, kk),
                    0x0004 => ld_vx_kk(x, kk),
                    0x0005 => ld_vx_kk(x, kk),
                    0x0006 => ld_vx_kk(x, kk),
                    0x0007 => ld_vx_kk(x, kk),
                    0x000E => ld_vx_kk(x, kk),
                    default => unreachable!(),
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

pub struct X(pub usize);
pub struct Y(pub usize);
struct KK(u8);
struct NNN(u12);
struct N(u16);

type Instruction = dyn Fn(&mut CPUState, &mut dyn ScreenDraw);

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
fn cls() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        screen_draw.clear();
        state.repaint.0 = true;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy0 - LD Vx, Vy</code></pre>
 * Set Vx = Vy
 */
fn ld_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>6xkk - LD Vx, kk</code></pre>
 * Set Vx = kk
 */
fn ld_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy1 - OR Vx, Vy</code></pre>
 * Set Vx = Vx OR Vy.
 */
fn or_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 | state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy2 - AND Vx, Vy</code></pre>
 * Set Vx = Vx AND Vy.
 */
fn and_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 & state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy3 - XOR Vx, Vy</code></pre>
 * Set Vx = Vx XOR Vy.
 */
fn xor_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 ^ state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy4 - ADD Vx, Vy</code></pre>
 * Set Vx = Vx + Vy, set VF = carry.
 */
fn add_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let sum = state.v[x.0].0 + state.v[y.0].0;
        let carry: u8 = match sum > 0xFF {
            true => 1,
            false => 0
        };
        state.v[0xF].0 = carry;
        state.v[x.0].0 = sum;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy5 - SUB Vx, Vy</code></pre>
 * Set Vx = Vx - Vy, set VF = NOT borrow.
 */
fn sub_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let not_borrow: u8 = match state.v[x.0].0 >= state.v[y.0].0 {
            true => 1,
            false => 0,
        };
        state.v[0xF].0 = not_borrow;
        state.v[x.0].0 = state.v[x.0].0 - state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy7 - SUBN Vx, Vy</code></pre>
 * Set Vx = Vy - Vx, set VF = NOT borrow.
 */
fn subn_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let not_borrow: u8 = match state.v[y.0].0 >= state.v[x.0].0 {
            true => 1,
            false => 0
        };
        state.v[0xF].0 = not_borrow;
        state.v[x.0].0 = state.v[y.0].0 - state.v[x.0].0;
        state.inc_pc_2();
    });
}


/**
 * <pre><code>8xy6 - SHR Vx, Vy</code></pre>
 * Set Vx = Vy SHR 1.
 * If shift quirks enabled Vx = Vx SHR 1.
 * If the least-significant bit of shifted value is 1, then VF is set to 1, otherwise 0.
 */
fn shr_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let y = match state.quirks.shift {
            true => x.0,
            false => y.0,
        };
        state.v[0xF].0 = state.v[y].0 & 0x01;
        state.v[x.0].0 = state.v[y].0 >> 1;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xyE - SHL Vx, Vy</code></pre>
 * Set Vx = Vy SHL 1.
 * If shift quirks enabled Vx = Vx SHL 1.
 * If the most-significant bit of shifted value is 1, then VF is set to 1, otherwise to 0.
 */
fn shl_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let y = match state.quirks.shift {
            true => x.0,
            false => y.0,
        };
        state.v[0xF].0 = (state.v[y].0 >> 7) & 0x01;
        state.v[x.0].0 = state.v[y].0 << 1;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>7xkk - ADD Vx, kk</code></pre>
 * Set Vx = Vx + kk.
 */
fn add_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 + kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>9xy0 - SNE Vx, Vy</code></pre>
 * Skip next instruction if Vx != Vy.
 */
fn sne_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        if state.v[x.0].0 != state.v[y.0].0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>4xkk - SNE Vx, kk</code></pre>
 * Skip next instruction if Vx != kk.
 */
fn sne_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        if state.v[x.0].0 != kk.0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>3xkk - SE Vx, kk</code></pre>
 * Skip next instruction if Vx = kk.
 */
fn se_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        if state.v[x.0].0 == kk.0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>5xy0 - SE Vx, Vy</code></pre>
 * Skip next instruction if Vx = Vy.
 */
fn se_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        if state.v[x.0].0 == state.v[y.0].0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>1nnn - JP nnn</code></pre>
 * Jump to location nnn.
 */
fn jp_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.pc.0 = nnn.0;
    });
}

/**
 * <pre><code>00EE - RET</code></pre>
 * Return from a subroutine.
 */
fn ret() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.sp.0 = (state.sp.0 - u4::new(1)) & u4::new((state.stack.len() - 1) as u8);
        state.pc.0 = state.stack[u8::from(state.sp.0) as usize];
        state.inc_pc_2();
    });
}

/**
 * <pre><code>2nnn - CALL nnn</code></pre>
 * Call subroutine at nnn.
 */
fn call_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.stack[u8::from(state.sp.0) as usize] = state.pc.0;
        state.sp.0 = (state.sp.0 + u4::new(1)) & u4::new((state.stack.len() - 1) as u8);
        state.pc.0 = nnn.0;
    });
}

/**
 * <pre><code>Annn - LD I, nnn</code></pre>
 * Set I = nnn.
 */
fn ld_i_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.I.0 = nnn.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Bnnn - JP V0, nnn</code></pre>
 * Jump to location nnn + V0.
 */
fn jp_v0_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.pc.0 = (nnn.0 + u12::new(state.v[0].0 as u16)) & u12::new(0x0FFF);
    });
}

/**
 * <pre><code>Cxkk - RND Vx, kk</code></pre>
 * Set Vx = random byte AND kk.
 */
fn rnd_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        let r: u8 = state.rng.gen(); // 0..255
        state.v[x.0].0 = r & kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx33 - LD B, Vx</code></pre>
 * Store BCD representation of Vx in memory locations I, I+1, and I+2.
 */
fn ld_b_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.mem[u16::from(state.I.0) as usize].0 = state.v[x.0].0 / 100;
        state.mem[u16::from(state.I.0 + u12::new(1)) as usize].0 = state.v[x.0].0 % 100 / 10;
        state.mem[u16::from(state.I.0 + u12::new(2)) as usize].0 = state.v[x.0].0 % 10;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx07 - LD Vx, DT</code></pre>
 * Set Vx = delay timer value.
 */
fn ld_vx_dt(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.dt.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx15 - LD DT, Vx</code></pre>
 * Set delay timer = Vx.
 */
fn ld_dt_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.dt.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx18 - LD ST, Vx</code></pre>
 * Set sound timer = Vx.
 */
fn ld_st_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.st.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx1E - ADD I, Vx</code></pre>
 * Set I = I + Vx.
 */
fn add_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.I.0 = (state.I.0 + u12::new(state.v[x.0].0 as u16));
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx55 - LD [I], Vx</code></pre>
 * Store registers V0 through Vx in memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
fn ld_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        for i in 0..=x.0 { // inclusive
            state.mem[u16::from(state.I.0) as usize + i].0 = state.v[i].0;
        }
        if !state.quirks.load_store {
            state.I.0 = state.I.0 + u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx65 - LD Vx, [I]</code></pre>
 * Read registers V0 through Vx from memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
fn ld_vx_i(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        for i in 0..=x.0 { // inclusive
            state.v[i].0 = state.mem[u16::from(state.I.0) as usize + i].0;
        }
        if !state.quirks.load_store {
            state.I.0 = u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx0A - LD Vx, K</code></pre>
 * Wait for a key press, store the value of the key in Vx.
 */
fn ld_vx_k(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.halted.0 = true;
        // TODO
        // cpu.keyboard.onNextKeyPressed = function (key) {
        //     state.v[x.0] = key;
        //     cpu.pc = (cpu.pc + 2) & 0x0FFF;
        //     cpu.halted = false;
        // };
    });
}

/**
 * <pre><code>Ex9E - SKP Vx</code></pre>
 * Skip next instruction if key with the value of Vx is pressed.
 */
fn skp_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        // TODO
        // if (cpu.keyboard.isKeyPressed(state.v[x.0])) {
        //     state.inc_pc_2();
        // }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>ExA1 - SKNP Vx</code></pre>
 * Skip next instruction if key with the value of Vx is not pressed.
 */
fn sknp_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        // TODO
        // if (!cpu.keyboard.isKeyPressed(state.v[x.0])) {
        //     cpu.pc = (cpu.pc + 2) & 0x0FFF;
        // }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx29 - LD F, Vx</code></pre>
 * Set I = location of sprite for digit Vx.
 */
fn ld_f_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.I = I(u12::new(state.v[x.0].0 as u16 * 5));
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Dxyn - DRW Vx, Vy, n</code></pre>
 * Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
 */
fn drw_vx_vy_n(x: X, y: Y, n: N) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[0xF] = V(0);
        for hline in 0..n.0 {
            let membyte = state.mem[u16::from(state.I.0 + u12::new(hline)) as usize];
            for vline in 0..8 {
                if (membyte.0 & (0x80 >> vline)) != 0 {
                    let nx = X(u16::from(state.v[x.0].0) as usize + vline);
                    let ny = Y(u16::from(state.v[y.0].0) as usize + hline as usize);
                    let coll = screen_draw.toggle_pixel(nx, ny);
                    if coll.0 {
                        state.v[0xF] = V(1);
                    }
                }
            }
        }
        state.repaint.0 = true;
        state.inc_pc_2();
    });
}
