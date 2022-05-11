Chip8 emulator - standalone (console output) and WASM lib

Build: `wasm-pack build --target web --out-dir pkg`

https://rustwasm.github.io/wasm-pack/book/tutorials/npm-browser-packages/packaging-and-publishing.html

The app exposes an init method and CPU type

```typescript
import initWasm, { CPU, init_program as initChip8 } from '@firfi/rust-wasm-chip8';

const init = async () => {

    await initWasm();
  
    const loadRom = async (rom: string) => {
        // load an existing ROM file from your server
        const arrayBuffer = await fetch(`roms/${rom}`).then(i => i.arrayBuffer());
        return new Uint8Array(arrayBuffer);
    }
    
    const romData = await loadRom("BLINKY");
    const canvas = document.getElementById("canvas");
    
    // the object will be moved each run(), hence "let"
    let chip8 = initChip8(romData, canvas);
    
    while (!chip8.is_done()) {
      chip8 = await chip8.run();
      // and calling chip8.stop() would set is_done() to true
      // also calling key_up and key_down between iterations would allow to control the emulator 
      // sic! all calls must be done on an actual instance of chip8 
    }
    
}
```
