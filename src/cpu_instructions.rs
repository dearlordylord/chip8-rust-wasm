use std::ops::Rem;
use ux::{u12, u4};

use crate::screen::{Screen, ScreenDraw};
use crate::cpu::{CPUState, I, V, PC, SP, DT, MemValue};

pub type Instruction = dyn Fn(&mut CPUState, &mut dyn Screen);

#[derive(Clone, Debug)]
pub struct X(pub usize);

impl Rem for X {
    type Output = X;

    fn rem(self, rhs: Self) -> Self::Output {
        X(self.0 % rhs.0)
    }
}

#[derive(Clone, Debug)]
pub struct Y(pub usize);

impl Rem for Y {
    type Output = Y;

    fn rem(self, rhs: Self) -> Self::Output {
        Y(self.0 % rhs.0)
    }
}

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
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
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
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn Screen| {
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
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
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
            y_val: V(0xBB),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
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
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.v[x.0].0 = kk.0;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_vx_kk() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x6000 | 9 << 8 | 0xFF,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(9),
            byte: V(0xFF),
            ..Default::default()
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.byte.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy1 - OR Vx, Vy</code></pre>
 * Set Vx = Vx OR Vy.
 */
pub fn or_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.v[x.0].0 = state.v[x.0].0 | state.v[y.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_or_vx_vy() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x8001 | 1 << 8 | 2 << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(1),
            y: Y(2),
            x_val: V(0xBB),
            y_val: V(0xCC),
            result: 0xFF,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy2 - AND Vx, Vy</code></pre>
 * Set Vx = Vx AND Vy.
 */
pub fn and_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.v[x.0].0 = state.v[x.0].0 & state.v[y.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_and_vx_vy() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x8002 | 2 << 8 | 3 << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(2),
            y: Y(3),
            x_val: V(0xCC),
            y_val: V(0xDD),
            result: 0xCC,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy3 - XOR Vx, Vy</code></pre>
 * Set Vx = Vx XOR Vy.
 */
pub fn xor_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.v[x.0].0 = state.v[x.0].0 ^ state.v[y.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_xor_vx_vy() {
    test_xor_vx_vy_inner(3, 4)
}

#[cfg(test)]
fn test_xor_vx_vy_inner(x: u16, y: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x8003 | x.clone() << 8 | y.clone() << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(0xDD),
            y_val: V(0xEE),
            result: 0x33,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy4 - ADD Vx, Vy</code></pre>
 * Set Vx = Vx + Vy, set VF = carry.
 */
pub fn add_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
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

#[test]
fn test_add_vx_vy() {
    test_add_vx_vy_inner(4, 5, 0x44, 0xAA, false, 0xEE);
    test_add_vx_vy_inner(4, 5, 0xAA, 0xAA, true, 0x54);
}

#[cfg(test)]
fn test_add_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, carry: bool, result: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x8004 | x.clone() << 8 | y.clone() << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(x_val),
            y_val: V(y_val),
            result,
            carry,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
            assert_eq!(state.v[0xF].0, match args.carry {
                true => 1,
                false => 0,
            });
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy5 - SUB Vx, Vy</code></pre>
 * Set Vx = Vx - Vy, set VF = NOT borrow.
 */
pub fn sub_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
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

#[test]
fn test_sub_vx_vy() {
    test_sub_vx_vy_inner(4, 5, 0xAA, 0x22, true, 0x88);
    test_sub_vx_vy_inner(4, 5, 0x22, 0xDD, false, 0x45);
    test_sub_vx_vy_inner(4, 5, 0x01, 0x01, true, 0x00); // error
}

#[cfg(test)]
fn test_sub_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, no_borrow: bool, result: u16) {
    test_subx_vx_vy_inner(x, y, x_val, y_val, no_borrow, result, 0x8005);
}

#[cfg(test)]
fn test_subx_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, no_borrow: bool, result: u16, op_code: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: op_code | x.clone() << 8 | y.clone() << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(x_val),
            y_val: V(y_val),
            result,
            no_borrow,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
            assert_eq!(state.v[0xF].0, match args.no_borrow {
                true => 1,
                false => 0,
            });
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>8xy7 - SUBN Vx, Vy</code></pre>
 * Set Vx = Vy - Vx, set VF = NOT borrow.
 */
pub fn subn_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
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

#[test]
fn test_subn_vx_vy() {
    test_subn_vx_vy_inner(6, 7, 0x22, 0xAA, true, 0x88);
    test_subn_vx_vy_inner(6, 7, 0xDD, 0x22, false, 0x45);
}

#[cfg(test)]
fn test_subn_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, no_borrow: bool, result: u16) {
    test_subx_vx_vy_inner(x, y, x_val, y_val, no_borrow, result, 0x8007);
}

/**
 * <pre><code>8xy6 - SHR Vx, Vy</code></pre>
 * Set Vx = Vy SHR 1.
 * If shift quirks enabled Vx = Vx SHR 1.
 * If the least-significant bit of shifted value is 1, then VF is set to 1, otherwise 0.
 */
pub fn shr_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        let y = match state.quirks.shift {
            true => x.0,
            false => y.0,
        };
        state.v[0xF].0 = state.v[y].0 & 0x01;
        state.v[x.0].0 = state.v[y].0 >> 1;
        state.inc_pc_2();
    });
}

#[test]
fn test_shr_vx_vy_quirks_disabled() {
    test_shr_vx_vy_inner(5, 6, 0x11, 0x44, 0x22, 0x44, 0x0, false);
    test_shr_vx_vy_inner(5, 6, 0x11, 0x45, 0x22, 0x45, 0x1, false);
}

#[test]
fn test_shr_vx_vy_quirks_enabled() {
    test_shr_vx_vy_inner(5, 6, 0x44, 0x11, 0x22, 0x11, 0x0, true);
    test_shr_vx_vy_inner(5, 6, 0x45, 0x11, 0x22, 0x11, 0x1, true);
}

#[cfg(test)]
fn test_shr_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, exp_x: u8, exp_y: u8, reg_f: u8, quirks_enabled: bool) {
    test_shx_vx_vy_inner(x, y, x_val, y_val, exp_x, exp_y, reg_f, quirks_enabled, 0x8006);
}

/**
 * <pre><code>8xyE - SHL Vx, Vy</code></pre>
 * Set Vx = Vy SHL 1.
 * If shift quirks enabled Vx = Vx SHL 1.
 * If the most-significant bit of shifted value is 1, then VF is set to 1, otherwise to 0.
 */
pub fn shl_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        let y = match state.quirks.shift {
            true => x.0,
            false => y.0,
        };
        state.v[0xF].0 = (state.v[y].0 >> 7) & 0x01;
        state.v[x.0].0 = state.v[y].0 << 1;
        state.inc_pc_2();
    });
}

#[test]
fn test_shl_vx_vy_quirks_disabled() {
    test_shl_vx_vy_inner(0x4, 0x5, 0x22, 0x44, 0x88, 0x44, 0x0, false);
    test_shl_vx_vy_inner(0x4, 0x5, 0x22, 0x45, 0x8A, 0x45, 0x0, false);
    test_shl_vx_vy_inner(0x4, 6, 0x22, 0xFF, 0xFE, 0xFF, 0x1, false);
    test_shl_vx_vy_inner(0x4, 7, 0x22, 0x7F, 0xFE, 0x7F, 0x0, false);
}

#[test]
fn test_shl_vx_vy_quirks_enabled() {
    test_shl_vx_vy_inner(0x4, 0x5, 0x44, 0x22, 0x88, 0x22, 0x0, true);
    test_shl_vx_vy_inner(0x4, 0x5, 0x45, 0x22, 0x8A, 0x22, 0x0, true);
    test_shl_vx_vy_inner(0x4, 6, 0xFF, 0x22, 0xFE, 0x22, 0x1, true);
    test_shl_vx_vy_inner(0x4, 7, 0x7F, 0x22, 0xFE, 0x22, 0x0, true);
}

#[cfg(test)]
fn test_shl_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, exp_x: u8, exp_y: u8, reg_f: u8, quirks_enabled: bool) {
    test_shx_vx_vy_inner(x, y, x_val, y_val, exp_x, exp_y, reg_f, quirks_enabled, 0x800E);
}

#[cfg(test)]
fn test_shx_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, exp_x: u8, exp_y: u8, reg_f: u8, quirks_enabled: bool, op_code: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: op_code | x.clone() << 8 | y.clone() << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(x_val),
            y_val: V(y_val),
            exp_x: V(exp_x),
            exp_y: V(exp_y),
            reg_f: V(reg_f),
            quirks_enabled,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.quirks.shift = args.quirks_enabled;
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.exp_x.0);
            assert_eq!(state.v[args.y.0].0, args.exp_y.0);
            assert_eq!(state.v[0xF].0, args.reg_f.0)
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>7xkk - ADD Vx, kk</code></pre>
 * Set Vx = Vx + kk.
 */
pub fn add_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        // game could overflow it
        state.v[x.0].0 = state.v[x.0].0.wrapping_add(kk.0);
        state.inc_pc_2();
    });
}

#[test]
fn test_add_vx_kk() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0x7000 | 0x8 << 8 | 0x55,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(0x8),
            x_val: V(0x30),
            result: 0x85,
            byte: V(0x55),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
        }),
        post_fn: Some(|state, _s, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.result as u8);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>9xy0 - SNE Vx, Vy</code></pre>
 * Skip next instruction if Vx != Vy.
 */
pub fn sne_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if state.v[x.0].0 != state.v[y.0].0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_sne_vx_vy() {
    test_sne_vx_vy_inner(6, 7, 10, 10, 2);
    test_sne_vx_vy_inner(6, 7, 10, 20, 4);
}

#[cfg(test)]
fn test_sne_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, pc_offset: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0x9000 | x << 8 | y << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(x_val),
            y_val: V(y_val),
            pc_offset: PC(u12::new(pc_offset)),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            // wrapping?  expect(cpu.pc).toEqual((this.oldPc + params.pcOffset) & 0x0FFF);
            let new_pc = scope.old_cpu_state.pc.0 + args.pc_offset.0;
            assert_eq!(state.pc.0, new_pc);
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>4xkk - SNE Vx, kk</code></pre>
 * Skip next instruction if Vx != kk.
 */
pub fn sne_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if state.v[x.0].0 != kk.0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_sne_vx_kk() {
    test_sne_vx_kk_inner(6, 10, 10, 2);
    test_sne_vx_kk_inner(8, 10, 60, 4);
}

#[cfg(test)]
fn test_sne_vx_kk_inner(x: u16, x_val: u8, byte: u8, pc_offset: u16) {
    test_snx_vx_kk_inner(x, x_val, byte, pc_offset, 0x4000);
}

#[cfg(test)]
fn test_snx_vx_kk_inner(x: u16, x_val: u8, byte: u8, pc_offset: u16, op_code: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: op_code | x << 8 | (byte as u16),
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            byte: V(byte),
            pc_offset: PC(u12::new(pc_offset)),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            let new_pc = scope.old_cpu_state.pc.0 + args.pc_offset.0;
            assert_eq!(state.pc.0, new_pc);
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>3xkk - SE Vx, kk</code></pre>
 * Skip next instruction if Vx = kk.
 */
pub fn se_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if state.v[x.0].0 == kk.0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_se_vx_kk() {
    test_se_vx_kk_inner(0xA, 0x60, 0x60, 4);
    test_se_vx_kk_inner(0xA, 0x30, 0x60, 2);
}

#[cfg(test)]
fn test_se_vx_kk_inner(x: u16, x_val: u8, byte: u8, pc_offset: u16) {
    test_snx_vx_kk_inner(x, x_val, byte, pc_offset, 0x3000);
}

/**
 * <pre><code>5xy0 - SE Vx, Vy</code></pre>
 * Skip next instruction if Vx = Vy.
 */
pub fn se_vx_vy(x: X, y: Y) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if state.v[x.0].0 == state.v[y.0].0 {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_se_vx_vy() {
    test_se_vx_vy_inner(8, 9, 20, 20, 4);
    test_se_vx_vy_inner(8, 9, 20, 40, 2);
}

#[cfg(test)]
fn test_se_vx_vy_inner(x: u16, y: u16, x_val: u8, y_val: u8, pc_offset: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0x5000 | x.clone() << 8 | y.clone() << 4,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            y: Y(y as usize),
            x_val: V(x_val),
            y_val: V(y_val),
            pc_offset: PC(u12::new(pc_offset)),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            cpu.state.v[args.y.0] = args.y_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            let new_pc = scope.old_cpu_state.pc.0 + args.pc_offset.0;
            assert_eq!(state.pc.0, new_pc);
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
            assert_eq!(state.v[args.y.0].0, args.y_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>1nnn - JP nnn</code></pre>
 * Jump to location nnn.
 */
pub fn jp_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.pc.0 = nnn.0;
    });
}

#[test]
fn test_jp_nnn() {
    test_jp_nnn_inner(0x0AAA);
    test_jp_nnn_inner(0x0FFF);
}

#[cfg(test)]
fn test_jp_nnn_inner(addr: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0x1000 | addr,
        op_args: Option::Some(TestCycleOpArgs {
            addr: PC(u12::new(addr)),
            ..Default::default()
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.pc.0, args.addr.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>00EE - RET</code></pre>
 * Return from a subroutine.
 */
pub fn ret() -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.pc.0 = state.stack[(u8::from(state.sp.0) - 1) as usize];
        state.sp.0 = state.sp.0 - u4::new(1);
        state.inc_pc_2();
    });
}

#[test]
fn test_ret() {
    test_ret_inner(vec![0x33], 1, 0x35);
    test_ret_inner(vec![0xAA, 0xBB, 0xCC], 3, 0xCE);
}

#[cfg(test)]
fn test_ret_inner(stack: Vec<u16>, sp: u8, expected_pc: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0x00EE,
        op_args: Option::Some(TestCycleOpArgs {
            addr: PC(u12::new(expected_pc)),
            stack: stack.iter().map(|x| u12::new(x.clone())).collect(),
            sp: SP(u4::new(sp)),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            for (i, x) in args.stack.iter().enumerate() {
                cpu.state.stack[i] = x.clone();
            }
            cpu.state.sp = args.sp;

        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.pc.0, args.addr.0);
            assert_eq!(state.sp.0, args.sp.0 - u4::new(1));
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>2nnn - CALL nnn</code></pre>
 * Call subroutine at nnn.
 */
pub fn call_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.stack[u8::from(state.sp.0) as usize] = state.pc.0;
        state.sp.0 = (state.sp.0 + u4::new(1)) & u4::new((state.stack.len() - 1) as u8);
        state.pc.0 = nnn.0;
    });
}

#[test]
fn test_call_nnn() {
    test_call_nnn_inner(0xAA);
    test_call_nnn_inner(0xBB);
    test_call_nnn_inner(0xCC);
}

#[cfg(test)]
fn test_call_nnn_inner(addr: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0x2000 | addr,
        op_args: Option::Some(TestCycleOpArgs {
            addr: PC(u12::new(addr)),
            ..Default::default()
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.pc.0, args.addr.0);
            assert_eq!(state.stack[u16::from(scope.old_cpu_state.sp.0) as usize], scope.old_cpu_state.pc.0);
            assert_eq!(state.sp.0, scope.old_cpu_state.sp.0 + u4::new(1));
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Annn - LD I, nnn</code></pre>
 * Set I = nnn.
 */
pub fn ld_i_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.i.0 = nnn.0;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_i_nnn() {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xA000 | 0xCCC,
        post_fn: Some(|state, scope, args| {
            assert_eq!(state.i.0, u12::new(0xCCC));
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Bnnn - JP V0, nnn</code></pre>
 * Jump to location nnn + V0.
 */
pub fn jp_v0_nnn(nnn: NNN) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        // not sure if wrapping add but https://github.com/mir3z/chip8-emu/blob/master/test/spec/is.spec.js would pass in that case
        state.pc.0 = (nnn.0.wrapping_add(u12::new(state.v[0].0 as u16)));
    });
}

#[test]
fn test_jp_v0_nnn() {
    test_jp_v0_nnn_inner(0x44, 0x555, 0x599);
    test_jp_v0_nnn_inner(0x1, 0xFFF, 0x000);
}

#[cfg(test)]
fn test_jp_v0_nnn_inner(v0: u8, addr: u16, expected_pc: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false,
        op_code: 0xB000 | addr,
        op_args: Option::Some(TestCycleOpArgs {
            addr: PC(u12::new(addr)),
            v0: V(v0),
            expected_pc: PC(u12::new(expected_pc)),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
           let args = args.unwrap();
            cpu.state.v[0] = args.v0;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.pc.0, args.expected_pc.0);
            assert_eq!(state.v[0].0, args.v0.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Cxkk - RND Vx, kk</code></pre>
 * Set Vx = random byte AND kk.
 */
pub fn rnd_vx_kk(x: X, kk: KK) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        let r: u8 = state.run_rng(); // 0..255
        state.v[x.0].0 = r & kk.0;
        state.inc_pc_2();
    });
}

#[test]
fn test_rnd_vx_kk() {
    test_rnd_vx_kk_inner(0xA);
}

#[cfg(test)]
fn test_rnd_vx_kk_inner(x: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xC000 | x << 8 | 1,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            ..Default::default()
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert!([0u8, 1u8].contains(&state.v[args.x.0].0));
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx33 - LD B, Vx</code></pre>
 * Store BCD representation of Vx in memory locations I, I+1, and I+2.
 */
pub fn ld_b_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.mem[u16::from(state.i.0) as usize].0 = state.v[x.0].0 / 100;
        state.mem[u16::from(state.i.0 + u12::new(1)) as usize].0 = state.v[x.0].0 % 100 / 10;
        state.mem[u16::from(state.i.0 + u12::new(2)) as usize].0 = state.v[x.0].0 % 10;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_b_vx() {
    test_ld_b_vx_inner(0xA, 123, 0x0AAA, vec![1, 2, 3]);
}

#[cfg(test)]
fn test_ld_b_vx_inner(x: u16, x_val: u8, i: u16, digits: Vec<u16>) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF033 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            i: I(u12::new(i)),
            digits: digits.iter().map(|d| d.clone() as u8).collect(),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.i = args.i;
            cpu.state.v[args.x.0] = args.x_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.i.0, args.i.0);
            let i = u16::from(state.i.clone().0) as usize;
            assert_eq!(state.mem[i].0, args.digits[0]);
            assert_eq!(state.mem[i + 1].0, args.digits[1]);
            assert_eq!(state.mem[i + 2].0, args.digits[2]);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx07 - LD Vx, DT</code></pre>
 * Set Vx = delay timer value.
 */
pub fn ld_vx_dt(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.v[x.0].0 = state.dt.0;
        state.inc_pc_2();
    });
}


#[test]
fn test_ld_vx_dt() {
    test_ld_vx_dt_inner(0xE, 0x20);
}

#[cfg(test)]
fn test_ld_vx_dt_inner(x: u16, dt: u8) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF007 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            dt: DT(dt),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.dt = args.dt;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.dt.0);
            assert_eq!(state.dt.0, args.dt.0 - 1);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx15 - LD DT, Vx</code></pre>
 * Set delay timer = Vx.
 */
pub fn ld_dt_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.dt.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_dt_vx() {
    test_ld_dt_vx_inner(0xD, 0x55);
}

#[cfg(test)]
fn test_ld_dt_vx_inner(x: u16, x_val: u8) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF015 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0 as usize] = args.x_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx18 - LD ST, Vx</code></pre>
 * Set sound timer = Vx.
 */
pub fn ld_st_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.st.0 = state.v[x.0].0;
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_st_vx() {
    test_ld_st_vx_inner(0xC, 0x66);
}

#[cfg(test)]
fn test_ld_st_vx_inner(x: u16, x_val: u8) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF018 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0 as usize] = args.x_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
            assert_eq!(state.st.0, args.x_val.0 - 1);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx1E - ADD I, Vx</code></pre>
 * Set I = I + Vx.
 */
pub fn add_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        // TODO wrapping?
        state.i.0 = state.i.0.wrapping_add(u12::new(state.v[x.0].0 as u16));
        state.inc_pc_2();
    });
}

#[test]
fn test_add_i_vx() {
    test_add_i_vx_inner(0xB, 0x33, 0x22, 0x55);
    test_add_i_vx_inner(0xA, 0xFF, 0x1, 0x100);
    test_add_i_vx_inner(0xC, 0x01, 0xFFF, 0);
}

#[cfg(test)]
fn test_add_i_vx_inner(x: u16, x_val: u8, i: u16, result: u16) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF01E | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            i: I(u12::new(i)),
            result,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.i.0 = args.i.0;
            cpu.state.v[args.x.0 as usize] = args.x_val;
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.i.0, u12::new(args.result));
            assert_eq!(state.v[args.x.0].0, args.x_val.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx55 - LD [I], Vx</code></pre>
 * Store registers V0 through Vx in memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
pub fn ld_i_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        for i in 0..=x.0 { // inclusive
            state.mem[u16::from(state.i.0) as usize + i].0 = state.v[i].0;
        }
        if !state.quirks.load_store {
            state.i.0 = state.i.0 + u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_i_vx_quirks_disabled() {
    test_ld_i_vx_inner(0x5, 0x0A00, 0x0A06, vec![0x5, 0x6, 0x7, 0x8, 0x9, 0xA], false);
    test_ld_i_vx_inner(0xF, 0x0A00, 0x0A10, vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF], false);
}

#[test]
fn test_ld_i_vx_quirks_enabled() {
    test_ld_i_vx_inner(0x5, 0x0A00, 0x0A00, vec![0x5, 0x6, 0x7, 0x8, 0x9, 0xA], true);
    test_ld_i_vx_inner(0xF, 0x0A00, 0x0A00, vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF], true);
}

#[cfg(test)]
fn test_ld_i_vx_inner(x: u16, i_val: u16, i_expected: u16, regs: Vec<u8>, quirks_enabled: bool) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF055 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            i_val: u12::new(i_val),
            i_expected: I(u12::new(i_expected)),
            regs: regs.iter().map(|r| V(r.clone())).collect(),
            quirks_enabled,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.i.0 = args.i_val;
            cpu.state.quirks.load_store = args.quirks_enabled;
            for (i, x) in args.regs.iter().enumerate() {
                cpu.state.v[i] = x.clone();
            }
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.i.0, args.i_expected.0);
            for (i, x) in (0..=args.x.0).enumerate() { // inclusive
                assert_eq!(state.mem[i + u16::from(args.i_val) as usize].0, state.v[i].0);
            }
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx65 - LD Vx, [I]</code></pre>
 * Read registers V0 through Vx from memory starting at location I.
 * The value of the I register will be incremented by X + 1, if load/store quirks are disabled.
 */
pub fn ld_vx_i(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        for i in 0..=x.0 { // inclusive
            state.v[i].0 = state.mem[u16::from(state.i.0) as usize + i].0;
        }
        if !state.quirks.load_store {
            state.i.0 = state.i.0 + u12::new(x.0 as u16) + u12::new(1);
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_ld_vx_i_quirks_disabled() {
    test_ld_vx_i_inner(0x6, 0x0B00, 0x0B07, vec![ 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9 ], false);
    test_ld_vx_i_inner(0xF, 0x0B00, 0x0B10, vec![ 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF ], false);
}

#[test]
fn test_ld_vx_i_quirks_enabled() {
    test_ld_vx_i_inner(0x6, 0x0B00, 0x0B00, vec![ 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9 ], true);
    test_ld_vx_i_inner(0xF, 0x0B00, 0x0B00, vec![ 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF ], true);
}

#[cfg(test)]
fn test_ld_vx_i_inner(x: u16, i_val: u16, i_expected: u16, mem: Vec<u8>, quirks_enabled: bool) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF065 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            i_val: u12::new(i_val),
            i_expected: I(u12::new(i_expected)),
            mem: mem.iter().map(|r| MemValue(r.clone())).collect(),
            quirks_enabled,
            ..Default::default()
        }),
        pre_fn: Some(|cpu, args| {
            let args = args.unwrap();
            cpu.state.i.0 = args.i_val;
            cpu.state.quirks.load_store = args.quirks_enabled;
            for (i, x) in args.mem.iter().enumerate() {
                cpu.state.mem[u16::from(cpu.state.i.0) as usize + i] = x.clone();
            }
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.i.0, args.i_expected.0);
            for (i, x) in (0..args.x.0).enumerate() { // inclusive
                assert_eq!(state.mem[i + u16::from(args.i_val) as usize].0, state.v[i].0);
            }
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx0A - LD Vx, K</code></pre>
 * Wait for a key press, store the value of the key in Vx.
 */
pub fn ld_vx_k(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.halted.0 = true;
        // TODO
        // cpu.keyboard.onNextKeyPressed = function (key) {
        // state.v[x.0] = V('6' as u8);
        // state.inc_pc_2();
        // state.halted.0 = false;
        // };
    });
}

#[test]
fn test_ld_vx_k() {
    let x = 0x5;
    let key = '6';
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        op_code: 0xF00A | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            key,
            ..Default::default()
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            assert_eq!(state.v[args.x.0].0, args.key as u8);
            assert!(!state.halted.0);
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Ex9E - SKP Vx</code></pre>
 * Skip next instruction if key with the value of Vx is pressed.
 */
pub fn skp_vx(_x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if state.keyboard.is_key_pressed(state.v[_x.0].0 as usize) {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_skp_vx() {
    test_skp_vx_inner(0x6, 0xA, true, 0xA as char, true);
    test_skp_vx_inner(0x6, 0xB, true, 0xA as char, false);
    test_skp_vx_inner(0x6, 0xC, false, 0xA as char, false);
}

#[cfg(test)]
fn test_skp_vx_inner(x: u16, x_val: u8, pressed: bool, key: char, should_skip: bool) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false, // test this explicitly
        op_code: 0xE09E | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            should_skip,
            pressed,
            key,
            ..Default::default()
        }),
        pre_fn: Some(move |cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            if (args.pressed) {
                cpu.state.keyboard.key_down(args.key as usize)
            }
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            if (args.should_skip) {
                assert_eq!(state.pc.0, scope.old_cpu_state.pc.0 + u12::new(4 as u16));
            } else {
                assert_eq!(state.pc.0, scope.old_cpu_state.pc.0 + u12::new(2 as u16));
            }
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>ExA1 - SKNP Vx</code></pre>
 * Skip next instruction if key with the value of Vx is not pressed.
 */
pub fn sknp_vx(_x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        if !state.keyboard.is_key_pressed(state.v[_x.0].0 as usize) {
            state.inc_pc_2();
        }
        state.inc_pc_2();
    });
}

#[test]
fn test_sknp_vx() {
    test_sknp_vx_inner(0x6, 0xA, false, 0xA as char, true);
    test_sknp_vx_inner(0x6, 0xB, true, 0xA as char, true);
    test_sknp_vx_inner(0x6, 0xB, true, 0xB as char, false);
}

#[cfg(test)]
fn test_sknp_vx_inner(x: u16, x_val: u8, pressed: bool, key: char, should_skip: bool) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false, // test this explicitly
        op_code: 0xE0A1 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            pressed,
            should_skip,
            key,
            ..Default::default()
        }),
        pre_fn: Some(move |cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
            if args.pressed {
                cpu.state.keyboard.key_down(args.key as usize)
            }
        }),
        post_fn: Some(|state, scope, args| {
            let args = args.unwrap();
            if args.should_skip {
                assert_eq!(state.pc.0, scope.old_cpu_state.pc.0 + u12::new(4 as u16));
            } else {
                assert_eq!(state.pc.0, scope.old_cpu_state.pc.0 + u12::new(2 as u16));
            }
        }),
        ..Default::default()
    });
}

/**
 * <pre><code>Fx29 - LD F, Vx</code></pre>
 * Set I = location of sprite for digit Vx.
 */
pub fn ld_f_vx(x: X) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, _screen_draw: &mut dyn Screen| {
        state.i = I(u12::new(state.v[x.0].0 as u16 * 5));
        state.inc_pc_2();
    });
}

#[cfg(test)]
fn test_ld_f_vx_inner(x: u16, x_val: u8, pressed: bool, should_skip: bool) {
    use super::test_utils::*;
    test_cycle(TestCycleParams {
        expect_inc: false, // test this explicitly
        op_code: 0xF029 | x << 8,
        op_args: Option::Some(TestCycleOpArgs {
            x: X(x as usize),
            x_val: V(x_val),
            ..Default::default()
        }),
        pre_fn: Some(move |cpu, args| {
            let args = args.unwrap();
            cpu.state.v[args.x.0] = args.x_val;
        }),
        // post_fn: Some(|state, scope, args| {
        //     // let args = args.unwrap();
        //     /*
        //     if (params.shouldSkip) {
        //         expect(cpu.pc).toEqual((this.oldPc + 4) & 0x0FFF);
        //     } else {
        //         expect(cpu.pc).toEqual((this.oldPc + 2) & 0x0FFF);
        //     }
        //      */
        // }),
        ..Default::default()
    });
}

/**
test_LD_F_Vx: function (cpu, params) {
            testCycle(cpu, {
                opCode: 0xF029 | (params.x << 8),
                op: 'LD_F_Vx',
                args: [ params.x ],
                preFn: function () {
                    cpu.V[params.x] = params.xVal;
                },
                postFn: function () {
                    expect(cpu.i).toEqual(params.i);
                    expect(cpu.pc).toEqual((this.oldPc + 2) & 0x0FFF);
                }
            });
        },
*/

/**
 * <pre><code>Dxyn - DRW Vx, Vy, n</code></pre>
 * Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
 */
pub fn drw_vx_vy_n(x: X, y: Y, n: N) -> Box<Instruction> {
    return Box::new(move |state: &mut CPUState, screen_draw: &mut dyn Screen| {
        state.v[0xF] = V(0);
        for hline in 0..n.0 {
            let membyte = state.mem[u16::from(state.i.0 + u12::new(hline)) as usize];
            for vline in 0..8 {
                if (membyte.0 & (0x80 >> vline)) != 0 {
                    let nx = X(u16::from(state.v[x.0].0) as usize + vline) % X(screen_draw.get_width());
                    let ny = Y(u16::from(state.v[y.0].0) as usize + hline as usize) % Y(screen_draw.get_height());
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
