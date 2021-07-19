use ux::{u12, u4};

use crate::screen::ScreenDraw;
use crate::cpu::{CPUState, I, V};

pub type Instruction = dyn Fn(&mut CPUState, &mut dyn ScreenDraw);

#[derive(Clone, Debug)]
pub struct X(pub usize);

#[derive(Clone, Debug)]
pub struct Y(pub usize);

#[derive(Clone, Debug)]
pub struct KK(pub u8);

#[derive(Clone, Debug)]
pub struct NNN(pub u12);

#[derive(Clone, Debug)]
pub struct N(pub u16);

/**
* <pre><code>0nnn - SYS addr</code></pre>
* Jump to a machine code routine at nnn.
* This instruction is only used on the old computers on which Chip-8 was originally implemented.
* It is ignored by modern interpreters.
*/
pub fn sys() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.inc_pc_2();
    });
}

#[test]
fn test_sys() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x0666,
        ..Default::default()
    })
}

/**
 * <pre><code>00E0 - CLS</code></pre>
 * Clears the display.
 */
pub fn cls() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        screen_draw.clear();
        state.repaint.0 = true;
        state.inc_pc_2();
    });
}

#[test]
fn test_cls() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x00E0,
        expectations: |s| {
            let screen_draw = s.screen_draw;
            screen_draw.expect_clear().return_const(());
            screen_draw.expect_repaint().return_const(());
        },
        ..Default::default()
    });
}

/**
 * <pre><code>8xy0 - LD Vx, Vy</code></pre>
 * Set Vx = Vy
 */
pub fn ld_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[y.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_vx_vy() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x8000 | 0 << 8 | 1 << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(0),
            y: Y(1),
            x_val: V(0xAA),
            y_val: V(0xBB)
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        expectations: |s| {

        },
        post_fn: Some(|state, s, args| {
            let args = args.unwrap();
            let state_vy = state.v[args.y.0].0.clone();
            assert_eq!(state.v[args.x.0].0, state_vy);
            assert_eq!(state_vy, args.y_val.0);
        }),
        ..Default::default()
    });
}
/**
 * <pre><code>6xkk - LD Vx, kk</code></pre>
 * Set Vx = kk
 */
pub fn ld_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy1 - OR Vx, Vy</code></pre>
 * Set Vx = Vx OR Vy.
 */
pub fn or_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 | state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy2 - AND Vx, Vy</code></pre>
 * Set Vx = Vx AND Vy.
 */
pub fn and_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 & state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy3 - XOR Vx, Vy</code></pre>
 * Set Vx = Vx XOR Vy.
 */
pub fn xor_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 ^ state.v[y.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy4 - ADD Vx, Vy</code></pre>
 * Set Vx = Vx + Vy, set VF = carry.
 */
pub fn add_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        let (sum, is_carry) = state.v[x.0].0.overflowing_add(state.v[y.0].0);
        let carry: u8 = match is_carry {
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
pub fn sub_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        let (sum, is_carry) = state.v[x.0].0.overflowing_sub(state.v[y.0].0);
        let not_borrow: u8 = match !is_carry {
            true => 1,
            false => 0,
        };
        state.v[0xF].0 = not_borrow;
        state.v[x.0].0 = sum;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>8xy7 - SUBN Vx, Vy</code></pre>
 * Set Vx = Vy - Vx, set VF = NOT borrow.
 */
pub fn subn_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        let (sum, is_carry) = state.v[y.0].0.overflowing_sub(state.v[x.0].0);
        let not_borrow: u8 = match !is_carry {
            true => 1,
            false => 0,
        };
        state.v[0xF].0 = not_borrow;
        state.v[x.0].0 = sum;
        state.inc_pc_2();
    });
}


/**
 * <pre><code>8xy6 - SHR Vx, Vy</code></pre>
 * Set Vx = Vy SHR 1.
 * If shift quirks enabled Vx = Vx SHR 1.
 * If the least-significant bit of shifted value is 1, then VF is set to 1, otherwise 0.
 */
pub fn shr_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn shl_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn add_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.v[x.0].0 + kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>9xy0 - SNE Vx, Vy</code></pre>
 * Skip next instruction if Vx != Vy.
 */
pub fn sne_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn sne_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn se_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn se_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn jp_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.pc.0 = nnn.0;
    });
}

/**
 * <pre><code>00EE - RET</code></pre>
 * Return from a subroutine.
 */
pub fn ret() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.sp.0 = (state.sp.0 - u4::new(1)) & u4::new((state.stack.len() - 1) as u8);
        state.pc.0 = state.stack[u8::from(state.sp.0) as usize];
        state.inc_pc_2();
    });
}

/**
 * <pre><code>2nnn - CALL nnn</code></pre>
 * Call subroutine at nnn.
 */
pub fn call_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.stack[u8::from(state.sp.0) as usize] = state.pc.0;
        state.sp.0 = (state.sp.0 + u4::new(1)) & u4::new((state.stack.len() - 1) as u8);
        state.pc.0 = nnn.0;
    });
}

/**
 * <pre><code>Annn - LD I, nnn</code></pre>
 * Set I = nnn.
 */
pub fn ld_i_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.i.0 = nnn.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Bnnn - JP V0, nnn</code></pre>
 * Jump to location nnn + V0.
 */
pub fn jp_v0_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.pc.0 = (nnn.0 + u12::new(state.v[0].0 as u16)) & u12::new(0x0FFF);
    });
}

/**
 * <pre><code>Cxkk - RND Vx, kk</code></pre>
 * Set Vx = random byte AND kk.
 */
pub fn rnd_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        let r: u8 = state.run_rng(); // 0..255
        state.v[x.0].0 = r & kk.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx33 - LD B, Vx</code></pre>
 * Store BCD representation of Vx in memory locations I, I+1, and I+2.
 */
pub fn ld_b_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.mem[u16::from(state.i.0) as usize].0 = state.v[x.0].0 / 100;
        state.mem[u16::from(state.i.0 + u12::new(1)) as usize].0 = state.v[x.0].0 % 100 / 10;
        state.mem[u16::from(state.i.0 + u12::new(2)) as usize].0 = state.v[x.0].0 % 10;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx07 - LD Vx, DT</code></pre>
 * Set Vx = delay timer value.
 */
pub fn ld_vx_dt(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.v[x.0].0 = state.dt.0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx15 - LD DT, Vx</code></pre>
 * Set delay timer = Vx.
 */
pub fn ld_dt_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.dt.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx18 - LD ST, Vx</code></pre>
 * Set sound timer = Vx.
 */
pub fn ld_st_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.st.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx1E - ADD I, Vx</code></pre>
 * Set I = I + Vx.
 */
pub fn add_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.i.0 = state.i.0 + u12::new(state.v[x.0].0 as u16);
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx55 - LD [I], Vx</code></pre>
 * Store registers V0 through Vx in memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
pub fn ld_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        for i in 0..=x.0 { // inclusive
            state.mem[u16::from(state.i.0) as usize + i].0 = state.v[i].0;
        }
        if !state.quirks.load_store {
            state.i.0 = state.i.0 + u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx65 - LD Vx, [I]</code></pre>
 * Read registers V0 through Vx from memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
pub fn ld_vx_i(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        for i in 0..=x.0 { // inclusive
            state.v[i].0 = state.mem[u16::from(state.i.0) as usize + i].0;
        }
        if !state.quirks.load_store {
            state.i.0 = u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Fx0A - LD Vx, K</code></pre>
 * Wait for a key press, store the value of the key in Vx.
 */
pub fn ld_vx_k(_x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn skp_vx(_x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn sknp_vx(_x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
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
pub fn ld_f_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn ScreenDraw| {
        state.i = I(u12::new(state.v[x.0].0 as u16 * 5));
        state.inc_pc_2();
    });
}

/**
 * <pre><code>Dxyn - DRW Vx, Vy, n</code></pre>
 * Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
 */
pub fn drw_vx_vy_n(x: X, y: Y, n: N) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn ScreenDraw| {
        state.v[0xF] = V(0);
        for hline in 0..n.0 {
            let membyte = state.mem[u16::from(state.i.0 + u12::new(hline)) as usize];
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
