<script lang="ts">
  import { spawn_event_loop, Proxy, Color, Palette, Options } from "DMG-2025";
  import { fade } from "svelte/transition";

  const inputMap: Record<string, string> = {
    Right: "ArrowRight",
    Left: "ArrowLeft",
    Up: "ArrowUp",
    Down: "ArrowDown",
    A: "x",
    B: "z",
    Select: "Backspace",
    Start: "Enter",
  };

  let currentPalette = $state("LCD");
  const palettes: Record<string, Palette> = {
    LCD: new Palette(
      new Color(0.7454042, 0.9386857, 0.6307571),
      new Color(0.2462013, 0.5271151, 0.1620293),
      new Color(0.0343398, 0.1384316, 0.0930589),
      new Color(0.0024282, 0.009134, 0.0144438),
    ),
    "Accurate LCD": new Palette(
      new Color(0.327778, 0.5028864, 0.0047769),
      new Color(0.2581828, 0.4125426, 0.0047769),
      new Color(0.0295568, 0.1221387, 0.0295568),
      new Color(0.0047769, 0.0395462, 0.0047769),
    ),
    Raw: new Palette(
      new Color(1.0, 1.0, 1.0),
      new Color(0.6666, 0.6666, 0.6666),
      new Color(0.3333, 0.3333, 0.3333),
      new Color(0.0, 0.0, 0.0),
    ),
  };

  function getOptions() {
    return new Options(palettes[currentPalette]);
  }
  let options = getOptions();

  let running = $state(false);
  let files: FileList | undefined = $state();
  $effect(() => {
    // Open selected file as byte array
    if (files) {
      files[0].arrayBuffer().then(load_rom);
    }
  });

  let proxy: Proxy | undefined = undefined;

  function resume() {
    // Progress emulator every animation frame for the duration it took to make last frame
    let lastTime = performance.now();
    function frame() {
      if (!running) {
        return;
      }
      let currentTime = performance.now();
      let millis = Math.min(17, Math.max(0, currentTime - lastTime));
      console.log(millis);
      lastTime = currentTime;

      proxy?.run_cpu(millis);
      window.requestAnimationFrame(frame);
    }
    window.requestAnimationFrame(frame);
  }

  function load_rom(rom: ArrayBuffer) {
    if (!proxy) {
      proxy = spawn_event_loop(options);
    }
    running = true;
    proxy.load_rom(new Uint8Array(rom));
    resume();
  }

  function handleKey(event: KeyboardEvent, pressed: boolean) {
    if (pressed && event.key === "Escape") {
      if (!proxy) {
        return;
      }
      running = !running;
      if (running) {
        resume();
      }
    }
    for (let key of Object.keys(inputMap)) {
      if (inputMap[key] === event.key) {
        console.log(event.key);
        proxy?.update_input(key, pressed);
      }
    }
  }

  function swapPalette() {
    let paletteNames = Object.keys(palettes);
    let paletteIndex = paletteNames.indexOf(currentPalette);
    paletteIndex++;
    if (paletteIndex >= paletteNames.length) {
      paletteIndex = 0;
    }
    currentPalette = paletteNames[paletteIndex];
    options = getOptions();
    proxy?.update_options(options);
  }
</script>

<svelte:window
  on:keydown={(event) => handleKey(event, true)}
  on:keyup={(event) => handleKey(event, false)}
/>

<main>
  <canvas id="canvas" tabindex="-1"></canvas>
  {#if !running}
    <div class="menu" transition:fade={{ duration: 100 }}>
      <input
        id="fileInput"
        accept=".gb"
        type="file"
        bind:files
        style="display: none"
      />
      <button onclick={() => document.getElementById("fileInput")?.click()}>
        Load ROM
      </button>
      <div class="palette">
        <p>Palette:</p>
        <button onclick={swapPalette}>{currentPalette}</button>
      </div>
    </div>
  {/if}
</main>
