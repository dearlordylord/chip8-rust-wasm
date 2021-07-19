use crate::screen::*;
use futures::{future::BoxFuture, future::ready};
use crate::cpu::{CPU, MemValue, CPUState, V};
use ux::u12;
use crate::cpu_instructions::{X, Y};

pub struct TestScreen<'a> {
    pub screen_draw: &'a mut MockScreenDraw,
}

impl<'a> Screen for TestScreen<'a> {
    fn request_animation_frame<'b>(&'b mut self) -> BoxFuture<'b, &'b mut dyn ScreenDraw> {
        // as &mut dyn ScreenDraw
        Box::pin(ready(self.screen_draw as &mut dyn ScreenDraw))
    }
}

pub struct TestScope<'a> {
    pub screen_draw: &'a mut MockScreenDraw,
    pub old_cpu_state: CPUState,
}

#[derive(Clone, Debug)]
pub(crate) struct TestCycleOpArgs {
    pub(crate) x: X,
    pub(crate) y: Y,
    pub(crate) x_val: V,
    pub(crate) y_val: V,
    pub(crate) byte: V,
    pub(crate) result: V,
}

impl Default for TestCycleOpArgs {
    fn default() -> Self {
        Self {
            x: X(0),
            y: Y(0),
            x_val: V(0),
            y_val: V(0),
            byte: V(0),
            result: V(0),
        }
    }
}

pub(crate) struct TestCycleParams {
    pub(crate) op_code: u16,
    pub(crate) op_args: Option<TestCycleOpArgs>,
    pub(crate) pre_fn: Option<fn(&mut CPU, Option<TestCycleOpArgs>)>,
    pub(crate) post_fn: Option<fn(&CPUState, TestScope, Option<TestCycleOpArgs>)>,
    pub(crate) op: Option<Box<fn()>>,
    pub(crate) expectations: fn(TestScope),
    pub(crate) expect_inc: bool,
}

impl Default for TestCycleParams {
    fn default() -> TestCycleParams {
        TestCycleParams {
            op_code: 0x00E0,
            op_args: Option::None,
            pre_fn: Option::None,
            post_fn: Option::None,
            op: Option::None,
            expectations: |_r| {},
            expect_inc: true,
        }
    }
}

pub(crate) fn test_cycle(params: TestCycleParams) {
    let mut screen_draw = MockScreenDraw::new();
    let screen = TestScreen { screen_draw: &mut screen_draw };
    let cpu = &mut CPU::new(Box::new(screen));
    cpu.state.mem[cpu.state.pci()] = MemValue(params.op_code.to_be_bytes()[0]);
    cpu.state.mem[cpu.state.pci() + 1] = MemValue(params.op_code.to_be_bytes()[1]);
    match params.pre_fn {
        Some(f) => f(cpu, params.op_args.clone()),
        None => {}
    }
    let mut screen_draw2 = MockScreenDraw::new();
    (params.expectations)(TestScope {
        screen_draw: &mut screen_draw2,
        old_cpu_state: cpu.state.clone(),
    });
    let old_pc = cpu.state.pc.0;
    CPU::step(&mut cpu.state, &mut screen_draw2).expect("expected to run successfully");
    cpu.state.update_timers();
    match params.post_fn {
        Some(f) => f(&cpu.state, TestScope {
            screen_draw: &mut screen_draw2,
            old_cpu_state: cpu.state.clone(),
        }, params.op_args.clone()),
        None => {}
    }
    assert_eq!(cpu.state.pc.0, old_pc + u12::new(match params.expect_inc {
        true => 2,
        false => 0
    }))
}