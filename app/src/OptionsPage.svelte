<script lang="ts">
  import Database from "./db.svelte";
  import FilePicker from "./FilePicker.svelte";
  import MenuSlider from "./MenuSlider.svelte";
  import { defaultOptions, type Options } from "./options.svelte";
  import { paletteNames } from "./options.svelte";

  let {
    options = $bindable(),
    db,
    successCallback,
    errorCallback,
  }: {
    options: Options;
    db: Database;
    successCallback: (msg: string) => void;
    errorCallback: (msg: string) => void;
  } = $props();

  let downloadElement: HTMLAnchorElement;
  let deletingData = $state(false);

  function readFileAsText(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (event) => {
        resolve(event.target?.result as string);
      };
      reader.onerror = (error) => {
        reject(error);
      };
      reader.readAsText(file);
    });
  }

  const importData = async (file: File) => {
    try {
      let json = await readFileAsText(file);
      await db.deserializeData(json);
      successCallback("Imported save data");
    } catch (e) {
      errorCallback(e as string);
    }
  };

  const exportData = async () => {
    try {
      // Serialize database
      let json = await db.serializeData();
      // Create a downloadable JSON file
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const date = new Date().toISOString();
      downloadElement.href = url;
      downloadElement.download = `DMG-2025-${date}.json`;
      // Start download
      downloadElement.click();
      // Remove URL object
      URL.revokeObjectURL(url);
    } catch (e) {
      errorCallback(e as string);
    }
  };

  const deleteData = () => {
    deletingData = false;
    db.deleteData()
      .then(() => successCallback("Deleted all save data :("))
      .catch(errorCallback);
  };

  function swapPalette() {
    if (options.paletteIndex == paletteNames.length - 1) {
      options.paletteIndex = 0;
    } else {
      options.paletteIndex++;
    }
  }
</script>

<div class="menu-grid" tabindex="-1">
  <p>Fast forward speed:</p>
  <MenuSlider
    bind:value={options.fast_forward_speed}
    values={[
      0.01, 0.05, 0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 1, 1.1, 1.3, 1.5, 2, 3, 5, 10,
      20,
    ]}
    labelFormatter={(value) => `${value}x`}
  />

  <p class="break"></p>
  <p>Scaling offset (zoom):</p>
  <MenuSlider bind:value={options.scaleOffset} min={-5} max={5} step={1} />
  <p>Color palette:</p>
  <button onclick={swapPalette}>{paletteNames[options.paletteIndex]}</button>
  <p>Background brightness:</p>
  <MenuSlider
    bind:value={options.ambientLight}
    labelFormatter={(value) => `${value}%`}
  />
  <p>UI transitions:</p>
  <button onclick={() => (options.uiTransitions = !options.uiTransitions)}>
    {options.uiTransitions ? "On" : "Off"}
  </button>

  <p class="break"></p>
  <p>Scanline strength:</p>
  <MenuSlider
    bind:value={options.scanlineStrength}
    labelFormatter={(value) => `${value}%`}
  />
  <p>Scanline size:</p>
  <MenuSlider
    bind:value={options.scanlineSize}
    min={0.01}
    max={0.5}
    step={0.01}
  />

  <p class="break"></p>
  <p>Glow:</p>
  <button onclick={() => (options.glowEnabled = !options.glowEnabled)}>
    {options.glowEnabled ? "Enabled" : "Disabled"}
  </button>
  <div
    class="menu-grid"
    style={"overflow-y: unset;" +
      (options.glowEnabled ? "" : "visibility: hidden;")}
  >
    <p>Background strength:</p>
    <MenuSlider
      bind:value={options.backgroundGlowStrength}
      labelFormatter={(value) => `${value}%`}
    />
    <p>Display strength:</p>
    <MenuSlider
      bind:value={options.displayGlowStrength}
      labelFormatter={(value) => `${value}%`}
    />
    <p>Quality:</p>
    <MenuSlider bind:value={options.glowQuality} min={1} max={10} step={1} />
    <p>Radius:</p>
    <MenuSlider bind:value={options.glowRadius} min={0.1} max={10} step={0.1} />
  </div>

  <p class="break"></p>
  <button class="danger-button" onclick={() => (options = defaultOptions)}>
    Reset options
  </button>

  <p class="break"></p>
  <a bind:this={downloadElement} style="display: none" href="placeholder">
    {""}
  </a>
  <div class="button-row">
    <FilePicker fileTypes={".json"} onPick={(file) => importData(file)}
      >Import save data</FilePicker
    >
    <button onclick={exportData}> Export save data </button>
  </div>
  {#if deletingData}
    <button class="danger-button" onclick={deleteData}>
      Click again to confirm
    </button>
  {:else}
    <button class="danger-button" onclick={() => (deletingData = true)}>
      Delete all save data
    </button>
  {/if}
</div>
