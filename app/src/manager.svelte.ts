
import { spawn_event_loop, Proxy, Options } from "DMG-2025";

export default class EmulatorManager {
  private proxy: Proxy | undefined = undefined;
  private lastFrameTime = 0;

  public initialized = false;
  public running = $state(false);
  public speed = $state(1);

  constructor(private options: Options) { }

  loadRom = (rom: ArrayBuffer) => {
    if (!this.initialized) {
      this.proxy = spawn_event_loop(this.options);
      this.initialized = true;
    }

    this.proxy?.load_rom(new Uint8Array(rom));
    this.toggle_execution();
  }

  toggle_execution = () => {
    if (!this.initialized) {
      throw new ReferenceError("Emulator is not initialized");
    }

    this.running = !this.running;
    if (this.running) {
      this.proxy?.set_audio_speed(this.speed);
      this.lastFrameTime = performance.now();
      window.requestAnimationFrame(this.frame);
    }
  }

  /**
   * Progresses emulator for the duration it took to make last frame
   */
  private frame = () => {
    if (!this.running) {
      return;
    }
    let currentTime = performance.now();
    let millis = Math.min(17, Math.max(0, currentTime - this.lastFrameTime));
    this.lastFrameTime = currentTime;

    this.proxy?.run_cpu(this.speed * millis);
    console.info(`Executed CPU for ${millis} ms`);
    window.requestAnimationFrame(this.frame);
  }

  updateOptions = (updater: (options: Options) => Options) => {
    let newOptions = updater(this.options);
    this.options = newOptions;
    this.proxy?.update_options(this.options);
  }

  updateInput = (key: string, pressed: boolean) => {
    this.proxy?.update_input(key, pressed);
  }
}
