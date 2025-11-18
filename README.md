![DMG-2025](/app/assets/logo_large.png)

## Features
- Reasonably accurate DMG (first Game Boy edition) emulation
  - Core written in _blazingly fast_ Rust, compiled to WebAssembly
  - Plays most games as on real hardware  
  - Passes `cpu_instrs`, `dmg-acid2` etc.
- GPU-based display rendering
  - Customizable color palettes
  - Configurable post-processing effects
- Integrated browser for ROMs from [Homebrew Hub](https://hh.gbdev.io/)
  - Load community made games and demos with just one click
- Save states
- Automatic saving of games whose cartridges support it 
- Fast forwarding / slowing down
- Input rebinding

## Local installation
- Install dependencies:
  - `npm`
  - `cargo`
  - `wasm-pack`
- Run dev server with `npm run dev`
- Create production build with `npm run build`
