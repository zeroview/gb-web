<script lang="ts">
  import Browser from "./Browser.svelte";
  import MenuSlider from "./MenuSlider.svelte";
  import EmulatorManager from "./manager.svelte";
  import { Color, Palette } from "DMG-2025";
  import { fade, fly } from "svelte/transition";
  import { onMount } from "svelte";

  let manager = new EmulatorManager();
  let browserVisible = $state(false);

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
      if (!manager.running) {
        browserVisible = false;
      }
    }
    for (let key of Object.keys(inputMap)) {
      if (inputMap[key] === event.key) {
        manager.updateInput(key, pressed);
      }
    }
  }

  const palettes: Record<string, Palette> = {
    LCD: new Palette(
      new Color(0.327778, 0.5028864, 0.0047769),
      new Color(0.2581828, 0.4125426, 0.0047769),
      new Color(0.0295568, 0.1221387, 0.0295568),
      new Color(0.0047769, 0.0395462, 0.0047769),
    ),
    Clear: new Palette(
      new Color(0.7454042, 0.9386857, 0.6307571),
      new Color(0.2462013, 0.5271151, 0.1620293),
      new Color(0.0343398, 0.1384316, 0.0930589),
      new Color(0.0024282, 0.009134, 0.0144438),
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
  let speedSliderVal = $state({
    value: speedSliderValues.indexOf(1),
    effect: (value: number) => {
      manager.options.speed = speedSliderValues[value];
      manager.updateOptions();
    },
  });

  let scaleSliderVal = $state({
    value: 0,
    effect: (val: number) => {
      manager.options.scale = val;
      manager.updateOptions();
    },
  });

  let volumeSliderVal = $state({
    value: 100,
    effect: (value: number) => {
      manager.options.volume = value / 100;
      manager.updateOptions();
    },
  });

  let backgroundGlowStrengthSliderVal = $state({
    value: 60,
    effect: (value: number) => {
      manager.options.background_glow_strength = value / 100;
      manager.updateOptions();
    },
  });

  let displayGlowStrengthSliderVal = $state({
    value: 30,
    effect: (value: number) => {
      manager.options.display_glow_strength = value / 100;
      manager.updateOptions();
    },
  });

  let glowQualitySliderVal = $state({
    value: 5,
    effect: (value: number) => {
      manager.options.glow_iterations = value * 2;
      manager.updateOptions();
    },
  });

  let glowRadiusSliderVal = $state({
    value: 0.5,
    effect: (value: number) => {
      manager.options.glow_radius = value;
      manager.updateOptions();
    },
  });

  let ambientLightSliderVal = $state({
    value: 0.3,
    effect: (value: number) => {
      manager.options.ambient_light = value;
      manager.updateOptions();
    },
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

  let popupVisible = $state(false);
  let popupText = $state("");
  function showMessage(msg: string) {
    popupText = msg;
    popupVisible = true;
    setTimeout(() => {
      popupVisible = false;
    }, 3000);
  }

  let eventListener: HTMLElement;
  onMount(() => {
    if (eventListener) {
      eventListener.addEventListener("romloaded", (e) => {
        const event = e as CustomEvent;
        document.title = `${event.detail} - DMG-2025`;
        console.info("Successfully loaded ROM");
        manager.toggle_execution();
      });
      eventListener.addEventListener("romloadfailed", (e) => {
        const event = e as CustomEvent;
        let msg = `Failed to load ROM: ${event.detail}`;
        console.error(msg);
        showMessage(msg);
      });
    }
  });
</script>

<svelte:window
  on:keydown={(event) => handleKey(event, true)}
  on:keyup={(event) => handleKey(event, false)}
/>

<main>
  <p id="eventListener" bind:this={eventListener}></p>
  {#if popupVisible}
    <p
      class="popup"
      in:fly={{ y: 30, duration: 300 }}
      out:fade={{ duration: 1000 }}
    >
      {popupText}
    </p>
  {/if}
  <canvas id="canvas" tabindex="-1"></canvas>
  {#if !manager.running}
    <div class="menu" transition:fade={{ duration: 100 }}>
      {#if browserVisible}
        <Browser {manager} onback={() => (browserVisible = false)} />
      {:else}
        <div class="menu-container">
          <input
            id="fileInput"
            accept=".gb,.zip"
            type="file"
            bind:files
            style="display: none"
          />
          <div class="button-row">
            <button
              onclick={() => document.getElementById("fileInput")?.click()}
            >
              Load ROM
            </button>
            <button onclick={() => (browserVisible = true)}
              >Browse Homebrew Hub</button
            >
          </div>
          <p style="height: 50px"></p>

          <MenuSlider
            value={speedSliderVal}
            min={0}
            max={speedSliderValues.length - 1}
            step={1}
            label="Speed:"
            valueLabelCallback={(value) => `${speedSliderValues[value]}x`}
          />
          <MenuSlider
            value={volumeSliderVal}
            min={0}
            max={200}
            step={1}
            label="Volume:"
            valueLabelCallback={(value) => `${value}%`}
          />
          <MenuSlider
            value={scaleSliderVal}
            min={-5}
            max={5}
            step={1}
            label="Scale offset:"
          />
          <div class="menu-row">
            <p style="text-align:right">Palette:</p>
            <button onclick={swapPalette}>{currentPalette}</button>
          </div>

          <p style="text-align:center; margin-top: 20px;">Glow Options</p>

          <MenuSlider
            value={backgroundGlowStrengthSliderVal}
            min={0}
            max={100}
            step={1}
            label="BG strength:"
            valueLabelCallback={(value) => `${value}%`}
          />
          <MenuSlider
            value={displayGlowStrengthSliderVal}
            min={0}
            max={100}
            step={1}
            label="Display strength:"
            valueLabelCallback={(value) => `${value}%`}
          />
          <MenuSlider
            value={glowQualitySliderVal}
            min={0}
            max={10}
            step={1}
            label="Quality:"
          />

          <MenuSlider
            value={glowRadiusSliderVal}
            min={0}
            max={5}
            step={0.1}
            label="Radius:"
          />
          <MenuSlider
            value={ambientLightSliderVal}
            min={0}
            max={1}
            step={0.01}
            label="Ambient light:"
          />
        </div>
      {/if}
    </div>
  {/if}
</main>
