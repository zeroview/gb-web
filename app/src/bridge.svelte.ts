import { spawn_event_loop, Proxy, type EmulatorOptions, ROMInfo } from "DMG-2025";
import type { Options } from "./options.svelte";
import { palettes, paletteNames } from "./options.svelte";

export default class EmulatorBridge {
  private proxy: Proxy | undefined = undefined;
  private lastFrameTime = 0;

  private speed = 0;
  public running = $state(false);

  initialize = (options: Options) => {
    this.proxy = spawn_event_loop();
    this.updateOptions(options);
    this.setSpeed(options.speed);
  }

  loadROM = async (rom: ArrayBuffer, isZip: boolean) => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }
    return this.proxy.query({ LoadROM: { file: new Uint8Array(rom), is_zip: isZip } }) as Promise<ROMInfo>
  }

  loadRAM = async (ram: Uint8Array) => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }
    return this.proxy.query({ LoadRAM: { ram } }) as Promise<void>;
  }

  toggle_execution = () => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }

    this.running = !this.running;
    this.proxy.query({ SetPaused: { paused: !this.running } })
    if (this.running) {
      this.lastFrameTime = performance.now();
      window.requestAnimationFrame(this.runEmulator);
    }
  }

  /**
   * Progresses emulator for the duration it took to make last frame
   */
  private runEmulator = () => {
    if (!this.running) {
      return;
    }
    let currentTime = performance.now();
    let millis = Math.min(17, Math.max(0, currentTime - this.lastFrameTime));
    this.lastFrameTime = currentTime;

    this.proxy?.query({ RunCPU: { millis: this.speed * millis } });
    console.info(`Executed CPU for ${millis} ms`);
    window.requestAnimationFrame(this.runEmulator);
  }

  saveRAM = async () => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }
    return this.proxy.query({ SaveRAM: {} }) as Promise<Uint8Array>;
  }

  serializeCPU = async () => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }
    return this.proxy.query({ SerializeCPU: {} }) as Promise<Uint8Array>;
  }

  deserializeCPU = async (buffer: Uint8Array) => {
    if (!this.proxy) {
      throw new ReferenceError("Emulator is not initialized");
    }
    return this.proxy.query({ DeserializeCPU: { buffer } }) as Promise<void>;
  }

  setSpeed = async (speed: number) => {
    this.speed = speed;
    if (!this.proxy) {
      return;
    }
    return this.proxy.query({ SetSpeed: { speed } }) as Promise<void>;
  }

  updateOptions = async (options: Options) => {
    if (!this.proxy) {
      return;
    }
    let emuOptions: EmulatorOptions = {
      palette: palettes[paletteNames[options.paletteIndex]],
      volume: options.volume / 100,
      scale_offset: options.scaleOffset,
      background_glow_strength: options.backgroundGlowStrength / 100,
      display_glow_strength: options.displayGlowStrength / 100,
      glow_iterations: options.glowQuality * 2,
      glow_radius: options.glowRadius,
      ambient_light: options.ambientLight,
    }
    return this.proxy.query({ UpdateOptions: { options: emuOptions } }) as Promise<void>;
  }

  updateInput = async (input: string, pressed: boolean) => {
    if (!this.proxy) {
      return;
    }
    return this.proxy.query({ UpdateInput: { input, pressed } }) as Promise<void>;
  }
}
