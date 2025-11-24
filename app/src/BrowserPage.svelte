<script lang="ts">
  import Fuse from "fuse.js";
  import MenuCheckbox from "./MenuCheckbox.svelte";
  import homebrewRoms from "../roms/homebrewhub.json";
  let {
    onLoadRom,
    onKeyboardFocus,
  }: {
    onLoadRom: (rom: ArrayBuffer, isZip: boolean) => void;
    onKeyboardFocus: (focus: boolean) => void;
  } = $props();
  interface ROMInfo {
    developer: string;
    typetag: string;
    download_url: string;
    image_url: string;
  }
  const roms = homebrewRoms as unknown as Record<string, ROMInfo>;
  let searchString = $state("");
  let games = $state(false);
  let demos = $state(false);
  let tools = $state(false);
  let music = $state(false);
  let romTitles = $derived.by(() => {
    const filtered = Object.keys(roms).filter((title) => {
      // Dont filter if filters arent enabled
      if (!(games || demos || tools || music)) {
        return true;
      }

      let typetag = roms[title].typetag;
      // Filter out not enabled ROM types
      return (
        !(!games && typetag == "game") &&
        !(!demos && typetag == "demo") &&
        !(!tools && typetag == "tool") &&
        !(!music && typetag == "music")
      );
    });
    if (!searchString) {
      return filtered;
    } else {
      const fuse = new Fuse(filtered);
      return fuse
        .search(searchString)
        .sort((a, b) => (a.score ?? 0) - (b.score ?? 0))
        .map((result) => result.item);
    }
  });

  function load(url: string) {
    fetch(url, { priority: "high" }).then((response) => {
      response.arrayBuffer().then((rom) => {
        onLoadRom(rom, false);
      });
    });
  }
</script>

<div class="browser-container">
  <div class="browser-topbar">
    <div class="browser-filters">
      <p>Filters:</p>
      <MenuCheckbox bind:value={games} />
      <p>Games</p>
      <MenuCheckbox bind:value={demos} />
      <p>Demos</p>
      <MenuCheckbox bind:value={tools} />
      <p>Tools</p>
      <MenuCheckbox bind:value={music} />
      <p>Music</p>
    </div>
    <input
      bind:value={searchString}
      placeholder="Search"
      onfocusin={() => onKeyboardFocus(true)}
      onfocusout={() => onKeyboardFocus(false)}
    />
  </div>
  <div class="browser-list" tabindex="-1">
    {#each romTitles as title}
      <div class="browser-item">
        <button
          class="browser-button"
          onclick={() => load(roms[title].download_url)}
        >
          <img
            src={roms[title].image_url ?? "/app/assets/cartridge.png"}
            alt={title}
            loading="lazy"
          />
          <div>
            <svg
              version="1.1"
              xmlns="http://www.w3.org/2000/svg"
              preserveAspectRatio="xMinYMin meet"
              viewBox="0 0 16 16"
            >
              <rect x="2" y="1" width="2" height="14" />
              <rect x="1" y="2" width="1" height="12" />
              <rect x="4" y="2" width="2" height="12" />
              <rect x="6" y="3" width="2" height="10" />
              <rect x="8" y="4" width="2" height="8" />
              <rect x="10" y="5" width="2" height="6" />
              <rect x="12" y="6" width="2" height="4" />
              <rect x="14" y="7" width="1" height="2" />
            </svg>
          </div>
        </button>

        <h3>{title}</h3>
        <p>{roms[title].developer}</p>
      </div>
    {/each}
  </div>
</div>
