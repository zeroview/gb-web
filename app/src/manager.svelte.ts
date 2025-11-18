import { spawn_event_loop, Proxy, EmulatorOptions, ProxyCallbacks, } from "DMG-2025";
import type { Options } from "./options.svelte";
import { palettes, paletteNames } from "./options.svelte";

export default class EmulatorManager {
  private proxy: Proxy | undefined = undefined;
  private callbacks: ProxyCallbacks = new ProxyCallbacks();
  private lastFrameTime = 0;

  private speed = 0;
  public initialized = false;
  public running = $state(false);

  initialize = (options: Options) => {
    this.proxy = spawn_event_loop();
    this.updateOptions(options);
    this.proxy.set_callbacks(this.callbacks);
    this.initialized = true;
  }

  loadRom = (rom: ArrayBuffer, isZip: boolean) => {
    if (!this.initialized) {
      throw new ReferenceError("Emulator is not initialized");
    }
    this.proxy?.load_rom(new Uint8Array(rom), isZip);
  }

  toggle_execution = () => {
    if (!this.initialized) {
      throw new ReferenceError("Emulator is not initialized");
    }

    this.running = !this.running;
    this.proxy?.set_paused(!this.running);
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

    this.proxy?.run_cpu(this.speed * millis);
    console.info(`Executed CPU for ${millis} ms`);
    window.requestAnimationFrame(this.runEmulator);
  }

  serializeCPU = () => {
    this.proxy?.serialize_cpu();
  }

  deserializeCPU = (buffer: Uint8Array) => {
    this.proxy?.deserialize_cpu(buffer);
  }

  setSpeed = (speed: number) => {
    this.speed = speed;
    this.proxy?.set_speed(speed);
  }

  updateOptions = (options: Options) => {
    let emuOptions = new EmulatorOptions();
    emuOptions.update_palette(palettes[paletteNames[options.paletteIndex]])
    emuOptions.volume = options.volume / 100;
    emuOptions.scale_offset = options.scaleOffset;
    emuOptions.background_glow_strength = options.backgroundGlowStrength / 100;
    emuOptions.display_glow_strength = options.displayGlowStrength / 100;
    emuOptions.glow_iterations = options.glowQuality * 2;
    emuOptions.glow_radius = options.glowRadius;
    emuOptions.ambient_light = options.ambientLight;
    this.proxy?.update_options(emuOptions);
  }

  updateInput = (key: string, pressed: boolean) => {
    this.proxy?.update_input(key, pressed);
  }

  onRomLoaded = (callback: (title: string, hash: number) => void) => {
    this.callbacks.set_rom_loaded(callback);
  }

  onCPUSerialization = (callback: (result: Uint8Array) => void) => {
    this.callbacks.set_cpu_serialized(callback);
  }

  onCPUDeserialization = (callback: () => void) => {
    this.callbacks.set_cpu_deserialized(callback);
  }

  onError = (callback: (error: string) => void) => {
    this.callbacks.set_error(callback);
  }
}
