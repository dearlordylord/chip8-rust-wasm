Chip8 emulator - standalone (console output) and WASM lib

Build: `wasm-pack build --scope firfi --target web`

Publish `cd pkg && npm publish --access=public`

https://rustwasm.github.io/wasm-pack/book/tutorials/npm-browser-packages/packaging-and-publishing.html

The app exposes an init method and CPU type

```typescript
import initWasm, { WasmProgram, init_program as initChip8 } from '@firfi/rust-wasm-chip8';


const init = async () => {

    await initWasm();
  
    const loadRom = async (rom: string) => {
        // load an existing ROM file from your server
        const arrayBuffer = await fetch(`roms/${rom}`).then(i => i.arrayBuffer());
        return new Uint8Array(arrayBuffer);
    }
    
    const romData = await loadRom("BLINKY");
    const canvas = document.getElementById("canvas");
    
    const cpu = initChip8(romData, canvas.getContext("2d"));
    cpu.run(); // will run asynchronously
    initKeyboardListeners(cpu); // wire up controls
    
}

const initKeyboardListeners = (cpu: WasmProgram) => {
  const makeCb = (kind: 'up' | 'down') => (e: KeyboardEvent) => {
    cpu[kind === 'up' ? 'key_up' : 'key_down'](e.which || e.keyCode);
  }
  const keyDownCb = makeCb('down');
  const keyUpCb = makeCb('up');
  document.addEventListener('keydown', keyDownCb);
  document.addEventListener('keyup', keyUpCb);
  return () => {
    document.removeEventListener('keydown', keyDownCb);
    document.removeEventListener('keyup', keyUpCb);
  };
}

```

Demo deployed on http://chip8-rust-wasm-frontend.apps.loskutoff.com

## TODO

use https://docs.rs/gloo-timers/latest/gloo_timers for timeouts
