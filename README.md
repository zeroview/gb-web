![gb-web](/app/assets/banner.png)

# Features
- Modern and responsive web UI with pixelated aesthetic
  - Written with Svelte, TypeScript and raw CSS
- Reasonably accurate emulation of the DMG (first Game Boy edition)
  - Core written in _blazingly fast_ Rust, compiled to WebAssembly
  - Plays most games as on real hardware  
  - Passes [`cpu_instrs`](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs), [`dmg-acid2`](https://github.com/mattcurrie/dmg-acid2) etc.
- GPU-based display rendering with WebGL
  - Customizable color palettes
  - Configurable shader effects
- Integrated browser for ROMs from [Homebrew Hub](https://hh.gbdev.io/)
  - Run over 800 community-made games and demos with just one click
- Saving games
  - Save states + automatic saving for cartridges that support it
  - Data is stored locally in IndexedDB, and can be exported in JSON 
- Fast-forward / slow-motion
- Input rebinding

# Contributing
Feel free to open issues: I will try my best to fix them or help with any problems. 

Pull requests are even more welcome, and I will be sure to check them out!

## Local installation
- Install development tools:
  - `node` and [`npm`](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) (or another package manager of your choice)
  - `rustc`, `cargo` and a C compiler ([further instructions](https://doc.rust-lang.org/book/ch01-01-installation.html))
  - [`wasm-pack`](https://drager.github.io/wasm-pack/installer/)
- Install WASM toolchain for Rust:
  ```console
  $ rustup target add wasm32-unknown-unknown --toolchain stable
  ```
- Install dependencies:
  ```console
  $ npm install
  ```
### Running
- Run development server:
  ```console
  $ npm run dev
  ```
- Create production build:
  ```console
  $ npm run build
  ```
