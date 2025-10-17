<script lang="ts">
  import EmulatorManager from "./manager.svelte";
  import { Color, Palette, Options } from "DMG-2025";
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

  let defaultPalette = Object.keys(palettes)[0];
  let currentPalette = $state(defaultPalette);
  let manager = new EmulatorManager(new Options(palettes[defaultPalette]));

  function handleKey(event: KeyboardEvent, pressed: boolean) {
    if (pressed && event.key === "Escape") {
      if (!manager.initialized) {
        return;
      }
      manager.toggle_execution();
    }
    for (let key of Object.keys(inputMap)) {
      if (inputMap[key] === event.key) {
        manager.updateInput(key, pressed);
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
    manager.updateOptions((options) => {
      options.palette = palettes[currentPalette];
      return options;
    });
  }

  let speedSliderVal = $state(0);
  $effect(() => {
    manager.speed = Number((10 ** speedSliderVal).toPrecision(2));
  });

  let files: FileList | undefined = $state();
  $effect(() => {
    // Open selected file as byte array
    if (files) {
      files[0].arrayBuffer().then(manager.loadRom);
    }
  });
</script>

<svelte:window
  on:keydown={(event) => handleKey(event, true)}
  on:keyup={(event) => handleKey(event, false)}
/>

<main>
  <canvas id="canvas" tabindex="-1"></canvas>
  {#if !manager.running}
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
      <div class="menu-row">
        <p>Palette:</p>
        <button onclick={swapPalette}>{currentPalette}</button>
      </div>
      <div class="menu-row">
        <p>Speed:</p>
        <input
          type="range"
          bind:value={speedSliderVal}
          min="-2"
          max="2"
          step="0.0001"
          style="width: 300px"
        />
        <p style="width: 60px">{manager.speed}</p>
      </div>
    </div>
  {/if}
</main>
