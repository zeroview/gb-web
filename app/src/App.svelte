<script lang="ts">
  import EmulatorManager from "./manager.svelte";
  import { Color, Palette } from "DMG-2025";
  import { fade } from "svelte/transition";

  let manager = new EmulatorManager();

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
  manager.options.update_palette(palettes[defaultPalette]);

  function swapPalette() {
    let paletteNames = Object.keys(palettes);
    let paletteIndex = paletteNames.indexOf(currentPalette);
    paletteIndex++;
    if (paletteIndex >= paletteNames.length) {
      paletteIndex = 0;
    }
    currentPalette = paletteNames[paletteIndex];

    manager.options.update_palette(palettes[currentPalette]);
    manager.updateOptions();
  }

  const speedSliderValues = [
    0.01, 0.05, 0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 1, 1.1, 1.3, 1.5, 2, 3, 5, 10, 20,
  ];
  let speedSliderVal = $state(speedSliderValues.indexOf(1));
  $effect(() => {
    manager.options.speed = speedSliderValues[speedSliderVal];
    manager.updateOptions();
  });

  let volumeSliderVal = $state(100);
  $effect(() => {
    manager.options.volume = volumeSliderVal / 100;
    manager.updateOptions();
  });

  const zipMimeTypes = [
    "application/zip",
    "application/x-zip-compressed",
    "application/x-zip",
  ];
  let files: FileList | undefined = $state();
  $effect(() => {
    // Open selected file as byte array
    if (files) {
      let file = files[0];
      let isZip = zipMimeTypes.includes(file.type);
      files[0].arrayBuffer().then((rom) => manager.loadRom(rom, isZip));
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
      <div class="menu-container">
        <input
          id="fileInput"
          accept=".gb,.zip"
          type="file"
          bind:files
          style="display: none"
        />
        <button onclick={() => document.getElementById("fileInput")?.click()}>
          Load ROM
        </button>
        <p style="height: 50px"></p>

        <div class="menu-row">
          <p style="text-align:right">Speed:</p>
          <input
            type="range"
            bind:value={speedSliderVal}
            min="0"
            max={speedSliderValues.length - 1}
            step="1"
            style="width: 250px"
          />
          <p>{`${speedSliderValues[speedSliderVal]}x`}</p>
        </div>

        <div class="menu-row">
          <p style="text-align:right">Volume:</p>
          <input
            type="range"
            bind:value={volumeSliderVal}
            min="0"
            max="200"
            step="1"
            style="width: 250px"
          />
          <p>{`${volumeSliderVal}%`}</p>
        </div>

        <div class="menu-row">
          <p style="text-align:right">Palette:</p>
          <button onclick={swapPalette}>{currentPalette}</button>
        </div>
      </div>
    </div>
  {/if}
</main>
