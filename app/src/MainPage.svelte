<script lang="ts">
  import EmulatorManager from "./manager.svelte";
  import { defaultOptions, type Options } from "./options.svelte";
  import MenuSlider from "./MenuSlider.svelte";

  let {
    manager,
    options = $bindable(),
    onBrowse,
    onSave,
    onLoad,
    onSaveSlotChange,
    saveDisabled,
    loadDisabled,
    saveSlot,
  }: {
    manager: EmulatorManager;
    options: Options;
    onBrowse: () => void;
    onSave: () => void;
    onLoad: () => void;
    onSaveSlotChange: (change: number) => void;
    saveDisabled: boolean;
    loadDisabled: boolean;
    saveSlot: number;
  } = $props();

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

  const speedSliderValues = [
    0.01, 0.05, 0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 1, 1.1, 1.3, 1.5, 2, 3, 5, 10, 20,
  ];
</script>

<input
  id="fileInput"
  accept=".gb,.zip"
  type="file"
  bind:files
  style="display: none"
/>
<div class="button-row">
  <button onclick={() => document.getElementById("fileInput")?.click()}>
    Load ROM
  </button>
  <button onclick={() => onBrowse()}>Browse Homebrew Hub</button>
</div>

<p style="height:1rem"></p>
<div class="button-row">
  <button onclick={onSave} disabled={saveDisabled}>Save state</button>
  <button onclick={onLoad} disabled={loadDisabled}>Load state</button>
  <div class="button-row" style="gap:1rem">
    <p>Slot:</p>
    <button onclick={() => onSaveSlotChange(-1)}>&lt;</button>
    <p style="width:2rem; text-align: center">{saveSlot}</p>
    <button onclick={() => onSaveSlotChange(1)}>&gt;</button>
  </div>
</div>
<p style="height:1.5rem"></p>
<div class="menu-grid">
  <p>Volume:</p>
  <MenuSlider
    bind:value={options.volume}
    min={0}
    max={200}
    step={5}
    valueLabelCallback={(value) => `${value}%`}
  />
  <p>Emulation speed:</p>
  <MenuSlider
    bind:value={
      () => speedSliderValues.indexOf(options.speed),
      (i) => (options.speed = speedSliderValues[i])
    }
    min={0}
    max={speedSliderValues.length - 1}
    step={1}
    valueLabelCallback={(value) => `${speedSliderValues[value]}x`}
  />

  <p>Fast forward speed:</p>
  <MenuSlider
    bind:value={
      () => speedSliderValues.indexOf(options.fast_forward_speed),
      (i) => (options.fast_forward_speed = speedSliderValues[i])
    }
    min={0}
    max={speedSliderValues.length - 1}
    step={1}
    valueLabelCallback={(value) => `${speedSliderValues[value]}x`}
  />

  <button class="danger-button" onclick={() => (options = defaultOptions)}
    >Reset options</button
  >
</div>
