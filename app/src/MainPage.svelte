<script lang="ts">
  import reloadIconUrl from "../assets/reload.png";
  import loadIconUrl from "../assets/load.png";
  import browseIconUrl from "../assets/browse.png";
  import playIconUrl from "../assets/play.png";

  import { type Options } from "./options.svelte";
  import FilePicker from "./FilePicker.svelte";
  import MenuSlider from "./MenuSlider.svelte";
  import type { LoadedROMInfo } from "./App.svelte";

  let {
    options = $bindable(),
    info,
    onBrowse,
    onSaveState,
    onLoadState,
    onSaveSlotChange,
    onLoadRom,
    onReload,
    onResume,
    romLoaded,
    loadStateDisabled,
    stateSlot,
  }: {
    options: Options;
    info: LoadedROMInfo;
    onBrowse: () => void;
    onSaveState: () => void;
    onLoadState: () => void;
    onSaveSlotChange: (change: number) => void;
    onLoadRom: (rom: ArrayBuffer, name: string, isZip: boolean) => void;
    onReload: () => void;
    onResume: () => void;
    romLoaded: boolean;
    loadStateDisabled: boolean;
    stateSlot: number;
  } = $props();

  const zipMimeTypes = [
    "application/zip",
    "application/x-zip-compressed",
    "application/x-zip",
  ];
</script>

{#if romLoaded}
  <button class="large img-button" onclick={onResume}>
    <img src={playIconUrl} alt="Resume" />
    <h2>{`Resume ${info.name}`}</h2>
  </button>
{:else}
  <h2>No cartridge inserted</h2>
{/if}
<div class="button-row load-buttons">
  {#if romLoaded}
    <button class="img-button" onclick={onReload}>
      <img src={reloadIconUrl} alt="Reload" />
      <p>Reload</p>
    </button>
  {/if}
  <FilePicker
    cssClass="img-button"
    fileTypes=".gb,.zip"
    onPick={(file) => {
      let isZip = zipMimeTypes.includes(file.type);
      file.arrayBuffer().then((rom) => onLoadRom(rom, file.name, isZip));
    }}
  >
    <img src={loadIconUrl} alt="Load" />
    <p>Load from disk</p>
  </FilePicker>
  <button class="img-button" onclick={() => onBrowse()}>
    <img src={browseIconUrl} alt="Browse" />
    <p>Browse Homebrew Hub</p>
  </button>
</div>

{#if romLoaded}
  <div class="button-row">
    <button onclick={onSaveState} disabled={!romLoaded}>Save state</button>
    <button onclick={onLoadState} disabled={loadStateDisabled}>
      Load state
    </button>
    <div class="button-row" style="flex-direction: row; gap: 1rem;">
      <p>Slot:</p>
      <button onclick={() => onSaveSlotChange(-1)}>&lt;</button>
      <p style="width:2rem; text-align: center">{stateSlot}</p>
      <button onclick={() => onSaveSlotChange(1)}>&gt;</button>
    </div>
  </div>
  <div class="menu-grid main-grid">
    <p>Volume:</p>
    <MenuSlider
      bind:value={options.volume}
      min={0}
      max={200}
      step={5}
      labelFormatter={(value) => `${value}%`}
    />
    <p>Emulation speed:</p>
    <MenuSlider
      bind:value={options.speed}
      values={[
        0.01, 0.05, 0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 1, 1.1, 1.3, 1.5, 2, 3, 5, 10,
        20,
      ]}
      labelFormatter={(value) => `${value}x`}
    />
  </div>
{/if}
