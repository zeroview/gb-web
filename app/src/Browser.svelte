<script lang="ts">
  import Fuse from "fuse.js";
  import EmulatorManager from "./manager.svelte";
  let { manager, onback }: { manager: EmulatorManager; onback: () => void } =
    $props();
  import homebrewRoms from "../roms/homebrewhub.json";

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
    fetch(url).then((response) => {
      response.arrayBuffer().then((buffer) => {
        manager.loadRom(buffer, false);
      });
    });
  }
</script>

<div class="browser-container">
  <div class="browser-topbar">
    <button
      style="white-space:nowrap"
      onclick={() => {
        onback();
      }}>{"< Back"}</button
    >
    <div class="browser-filters">
      <p>Filters:</p>
      <input type="checkbox" bind:checked={games} />
      <p>Games</p>
      <input type="checkbox" bind:checked={demos} />
      <p>Demos</p>
      <input type="checkbox" bind:checked={tools} />
      <p>Tools</p>
      <input type="checkbox" bind:checked={music} />
      <p>Music</p>
    </div>
    <input bind:value={searchString} placeholder="Search" />
  </div>
  <div class="browser-list">
    {#each romTitles as title}
      <div class="browser-item">
        {#if roms[title].image_url !== ""}
          <button
            class="browser-img"
            onclick={() => load(roms[title].download_url)}
          >
            <img src={roms[title].image_url} alt={title} />
            <div class="browser-img-overlay">
              <p class="browser-img-overlay-text">▶</p>
            </div>
          </button>
        {/if}

        <h3>{title}</h3>
        <p>{roms[title].developer}</p>
      </div>
    {/each}
  </div>
</div>
