use crate::screen::*;
use futures::{future::BoxFuture, future::ready};
use crate::cpu::{CPU, MemValue, CPUState};
use ux::u12;

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

pub struct TestCycleParams {
    pub op_code: u16,
    pub pre_fn: Option<Box<fn(TestScope)>>,
    pub post_fn: Option<Box<fn(TestScope)>>,
    pub op: Option<Box<fn()>>,
    pub expectations: fn(TestScope),
    pub expect_inc: bool,
}

impl Default for TestCycleParams {
    fn default() -> TestCycleParams {
        TestCycleParams {
            op_code: 0x00E0,
            pre_fn: Option::None,
            post_fn: Option::None,
            op: Option::None,
            expectations: |_r| {},
            expect_inc: true,
        }
    }
}

pub fn test_cycle(params: TestCycleParams) {
    let mut screen_draw = MockScreenDraw::new();
    let screen = TestScreen { screen_draw: &mut screen_draw };
    let cpu = &mut CPU::new(Box::new(screen));
    // cpu.state.mem[cpu.state.pci()] = MemValue(0x00E0_u16.to_be_bytes()[0]);
    cpu.state.mem[cpu.state.pci()] = MemValue(params.op_code.to_be_bytes()[0]);
    // cpu.state.mem[cpu.state.pci() + 1] = MemValue(0x00E0_u16.to_be_bytes()[1]);
    cpu.state.mem[cpu.state.pci() + 1] = MemValue(params.op_code.to_be_bytes()[1]);
    let mut screen_draw2 = MockScreenDraw::new();
    (params.expectations)(TestScope {
        screen_draw: &mut screen_draw2,
        old_cpu_state: cpu.state.clone(),
    });
    let old_pc = cpu.state.pc.0;
    CPU::step(&mut cpu.state, &mut screen_draw2).expect("expected to run successfully");
    cpu.state.update_timers();
    assert_eq!(cpu.state.pc.0, old_pc + u12::new(match params.expect_inc {
        true => 2,
        false => 0
    }))
}