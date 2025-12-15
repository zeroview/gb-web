<script lang="ts">
  import cartridgeImageUrl from "../assets/cartridge.png";
  import playIconUrl from "../assets/play.png";
  import loadingAnimationUrl from "../assets/loading.gif";

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
    onLoadRom: (rom: ArrayBuffer, name: string, isZip: boolean) => void;
    onKeyboardFocus: (focus: boolean) => void;
  } = $props();
  const roms = homebrewRoms as unknown as Record<string, BrowserROMInfo>;

  let romTitles = $derived.by(() => {
    // Filter ROM list based on checkboxes
    const filtered = Object.keys(roms).filter((title) => {
      /// Filter featured games
      if (filters.featured && !featuredTitles.includes(title)) {
        return false;
      }
      // Dont filter if filters arent enabled
      if (!(filters.games || filters.demos || filters.tools || filters.music)) {
        return true;
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

  let loading = $state(false);
  function load(url: string, name: string) {
    if (loading) {
      return;
    }
    loading = true;
    fetch(url, { priority: "high" }).then((response) => {
      response.arrayBuffer().then((rom) => {
        onLoadRom(rom, name, false);
      });
    });
  }
</script>

<div class="browser-container">
  <div class="browser-topbar">
    <div class="browser-filters">
      <MenuCheckbox
        bind:value={filters.featured}
        label="Featured"
        featured={true}
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
          onclick={() => load(roms[title].download_url, title)}
        >
          <img
            src={roms[title].image_url ?? cartridgeImageUrl}
            alt={title}
            loading="lazy"
            fetchpriority="low"
          />
          <div>
            <img src={loading ? loadingAnimationUrl : playIconUrl} alt="Play" />
          </div>
        </button>

        <h3>{title}</h3>
        <p>{roms[title].developer}</p>
      </div>
    {/each}
  </div>
</div>
