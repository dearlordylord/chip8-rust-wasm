mod cpu;
mod screen;
mod console_screen;
mod macros;
mod cpu_decoder;
mod cpu_instructions;

use std::fs::File;
use std::io::Read;
use cpu::CPU;
use crate::console_screen::ConsoleScreen;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let program = read_rom();
    let mut cpu = CPU::new(Box::new(ConsoleScreen::new()));
    cpu.load_program(program);
    cpu.run().await;
    Ok(())
}

fn read_rom() -> Vec<u8> {
    let mut file = File::open("BLINKY").unwrap();
    let mut r: Vec<u8> = Vec::new();
    file.read_to_end(&mut r).unwrap();
    return r;
}

#[cfg(test)]
mod tests {
    use crate::screen::*;
    use futures::{future::BoxFuture, future::ready};
    use crate::cpu::{CPU, MemValue};
    use ux::u12;

    struct TestScreen<'a> {
        screen_draw: &'a mut MockScreenDraw,
    }

    impl<'a> Screen for TestScreen<'a> {
        fn request_animation_frame<'b>(&'b mut self) -> BoxFuture<'b, &'b mut dyn ScreenDraw> {
            // as &mut dyn ScreenDraw
            Box::pin(ready(self.screen_draw as &mut dyn ScreenDraw))
        }
    }

    /*
      spyOn(cpu.screen, 'repaint');
            spyOn(cpu.screen, 'clear');

            testCycle(cpu, {
                opCode: 0x00E0,
                op: 'CLS',
                args: [ ],
                postFn: function () {
                    expect(cpu.pc).toEqual((this.oldPc + 2) & 0x0FFF);
                    expect(cpu.screen.clear).toHaveBeenCalled();
                    expect(cpu.screen.repaint).toHaveBeenCalled();
                }
            });
     */

    #[test]
    fn it_works() {
        let mut screen_draw = MockScreenDraw::new();
        let screen = TestScreen { screen_draw: &mut screen_draw };
        let cpu = &mut CPU::new(Box::new(screen));
        cpu.state.mem[cpu.state.pci()] = MemValue(0x00E0_u16.to_be_bytes()[0]);
        cpu.state.mem[cpu.state.pci() + 1] = MemValue(0x00E0_u16.to_be_bytes()[1]);
        let mut screen_draw2 = MockScreenDraw::new();
        screen_draw2.expect_clear().return_const(());
        screen_draw2.expect_repaint().return_const(());
        let old_pc = cpu.state.pc.0;
        CPU::step(&mut cpu.state, &mut screen_draw2).expect("expected to run successfully");
        cpu.state.update_timers();
        assert_eq!(cpu.state.pc.0, old_pc + u12::new(2))
    }
}