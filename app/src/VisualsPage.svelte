<script lang="ts">
  import MenuSlider from "./MenuSlider.svelte";
  import type { Options } from "./options.svelte";
  import { paletteNames } from "./options.svelte";

  let { options = $bindable() }: { options: Options } = $props();

  function swapPalette() {
    if (options.paletteIndex == paletteNames.length - 1) {
      options.paletteIndex = 0;
    } else {
      options.paletteIndex++;
    }
  }
</script>

<div class="menu-grid">
  <p>Scaling offset:</p>
  <MenuSlider bind:value={options.scaleOffset} min={-5} max={5} step={1} />
  <p>Color palette:</p>
  <button onclick={swapPalette}>{paletteNames[options.paletteIndex]}</button>
  <p>Background brightness:</p>
  <MenuSlider
    bind:value={options.ambientLight}
    min={0}
    max={100}
    step={1}
    valueLabelCallback={(value) => `${value}%`}
  />
  <p>UI transitions:</p>
  <button onclick={() => (options.uiTransitions = !options.uiTransitions)}>
    {options.uiTransitions ? "On" : "Off"}
  </button>

  <p style="grid-column: span 2; height: 2rem"></p>
  <p>Scanline strength:</p>
  <MenuSlider
    bind:value={options.scanlineStrength}
    min={0}
    max={100}
    step={1}
    valueLabelCallback={(value) => `${value}%`}
  />
  <p>Scanline size:</p>
  <MenuSlider bind:value={options.scanlineSize} min={0.01} max={0.5} step={0.01}
  ></MenuSlider>

  <p style="grid-column: span 2; height: 2rem"></p>
  <p>Glow:</p>
  <button onclick={() => (options.glowEnabled = !options.glowEnabled)}>
    {options.glowEnabled ? "Enabled" : "Disabled"}
  </button>
  <div
    class="menu-grid"
    style={options.glowEnabled ? "" : "visibility:hidden;"}
  >
    <p>Background strength:</p>
    <MenuSlider
      bind:value={options.backgroundGlowStrength}
      min={0}
      max={100}
      step={1}
      valueLabelCallback={(value) => `${value}%`}
    />
    <p>Display strength:</p>
    <MenuSlider
      bind:value={options.displayGlowStrength}
      min={0}
      max={100}
      step={1}
      valueLabelCallback={(value) => `${value}%`}
    />
    <p>Quality:</p>
    <MenuSlider bind:value={options.glowQuality} min={1} max={10} step={1} />
    <p>Radius:</p>
    <MenuSlider bind:value={options.glowRadius} min={0.1} max={10} step={0.1} />
  </div>
</div>
