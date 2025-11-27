<script lang="ts">
  import Fuse from "fuse.js";
  import MenuCheckbox from "./MenuCheckbox.svelte";
  import homebrewRoms from "../roms/homebrewhub.json";
  import featuredTitles from "../roms/featured.json";

  /// Info about a ROM in the browser
  interface BrowserROMInfo {
    developer: string;
    typetag: string;
    download_url: string;
    image_url: string;
  }

  interface BrowserFilters {
    search: string;
    featured: boolean;
    games: boolean;
    demos: boolean;
    tools: boolean;
    music: boolean;
  }

  let {
    filters = $bindable(),
    onLoadRom,
    onKeyboardFocus,
  }: {
    filters: BrowserFilters;
    onLoadRom: (rom: ArrayBuffer, isZip: boolean) => void;
    onKeyboardFocus: (focus: boolean) => void;
  } = $props();
  const roms = homebrewRoms as unknown as Record<string, BrowserROMInfo>;

  let romTitles = $derived.by(() => {
    // Filter ROM list based on checkboxes
    const filtered = Object.keys(roms).filter((title) => {
      // Dont filter if filters arent enabled
      if (
        !(
          filters.featured ||
          filters.games ||
          filters.demos ||
          filters.tools ||
          filters.music
        )
      ) {
        return true;
      }
      /// Filter featured games
      if (filters.featured) {
        return featuredTitles.includes(title);
      }
      let typetag = roms[title].typetag;
      // Filter out not enabled ROM types based on typetag string
      return (
        !(!filters.games && typetag == "game") &&
        !(!filters.demos && typetag == "demo") &&
        !(!filters.tools && typetag == "tool") &&
        !(!filters.music && typetag == "music")
      );
    });
    /// Search ROM titles with Fuse
    if (!filters.search) {
      return filtered;
    } else {
      const fuse = new Fuse(filtered);
      return fuse
        .search(filters.search)
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
      <MenuCheckbox
        --main-color="#d1c554"
        --active-color="#ffe500"
        bind:value={filters.featured}
        label="Featured"
      />
      <MenuCheckbox bind:value={filters.games} label="Games" />
      <MenuCheckbox bind:value={filters.demos} label="Demos" />
      <MenuCheckbox bind:value={filters.tools} label="Tools" />
      <MenuCheckbox bind:value={filters.music} label="Music" />
    </div>
    <input
      bind:value={filters.search}
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
