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
  <h3>General</h3>
  <p>Scaling offset:</p>
  <MenuSlider bind:value={options.scaleOffset} min={-5} max={5} step={1} />
  <p>Color palette:</p>
  <button onclick={swapPalette}>{paletteNames[options.paletteIndex]}</button>
  <p>UI transitions:</p>
  <button onclick={() => (options.uiTransitions = !options.uiTransitions)}>
    {options.uiTransitions ? "On" : "Off"}
  </button>

  <h3>Glow</h3>
  <p>BG strength:</p>
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
  <MenuSlider bind:value={options.glowQuality} min={0} max={10} step={1} />
  <p>Radius:</p>
  <MenuSlider bind:value={options.glowRadius} min={0} max={5} step={0.1} />
  <p>Ambient light:</p>
  <MenuSlider bind:value={options.ambientLight} min={0} max={1} step={0.01} />
</div>
