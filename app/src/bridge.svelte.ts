import type { ROMInfo, Proxy } from "wasm";
import type { Options } from "./options.svelte";
import { toEmulatorOptions } from "./options.svelte";

export default class EmulatorBridge {
  private proxy: Proxy | undefined = undefined;
  private lastFrameTime = 0;

  private speed = 0;
  private maxFrameTime: number = 0;
  public initialized = $state(false);
  public running = $state(false);
  public showOnscreenControls: boolean = false;

  initialize = async (options: Options) => {
    const wasm = await import("wasm");
    this.proxy = wasm.spawn_event_loop();
    this.updateOptions(options);
    this.setSpeed(options.speed);
    this.initialized = true;
  }

  loadROM = async (rom: ArrayBuffer, isZip: boolean) => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
    }
    return this.proxy.query({ LoadROM: { file: new Uint8Array(rom), is_zip: isZip } }) as Promise<ROMInfo>;
  }

  reload = async () => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
    }
    return this.proxy.query({ Reload: {} }) as Promise<void>;
  }

  loadRAM = async (ram: Uint8Array) => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
    }
    return this.proxy.query({ LoadRAM: { ram } }) as Promise<void>;
  }

  toggle_execution = () => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
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
    let timeToExecute = Math.min(this.maxFrameTime, Math.max(0, currentTime - this.lastFrameTime));
    this.lastFrameTime = currentTime;

    console.info(`Queried CPU to execute for ${timeToExecute} ms`);
    this.proxy?.query({ RunCPU: { millis: this.speed * timeToExecute } }).then(() => {
      let executionTime = performance.now() - currentTime;
      console.info(`CPU took ${executionTime} ms to execute`);
    });
    window.requestAnimationFrame(this.runEmulator);
  }

  saveRAM = async () => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
    }
    return this.proxy.query({ SaveRAM: {} }) as Promise<Uint8Array>;
  }

  serializeCPU = async () => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
    }
    return this.proxy.query({ SerializeCPU: {} }) as Promise<Uint8Array>;
  }

  deserializeCPU = async (buffer: Uint8Array) => {
    if (!this.proxy) {
      throw new Error("Emulator is not initialized");
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
    this.maxFrameTime = (1 / options.fpsTarget) * 1000;
    return this.proxy.query({ UpdateOptions: { options: toEmulatorOptions(options, this.showOnscreenControls) } }) as Promise<void>;
  }

  updateInput = async (input: string, pressed: boolean) => {
    if (!this.proxy) {
      return;
    }
    return this.proxy.query({ UpdateInput: { input, pressed } }) as Promise<void>;
  }

  updatePointerPos = async (id: number, x: number, y: number) => {
    if (!this.proxy) {
      return;
    }
    return this.proxy.query({ UpdatePointerPos: { id, pos: [x, y] } }) as Promise<void>;
  }

  updatePointerPressed = async (id: number, pressed: boolean) => {
    if (!this.proxy) {
      return;
    }

    return this.proxy.query({ UpdatePointerPressed: { id, pressed } }) as Promise<void>;
  }
}
