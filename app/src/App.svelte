<script lang="ts">
  import MainPage from "./MainPage.svelte";
  import BrowserPage from "./BrowserPage.svelte";
  import VisualsPage from "./VisualsPage.svelte";
  import InputPage from "./InputPage.svelte";

  import InputManager from "./input.svelte";
  import EmulatorManager from "./manager.svelte";

  import { fade, fly } from "svelte/transition";
  import { loadOptions, saveOptions } from "./options.svelte";
  import { onMount } from "svelte";

  let manager = new EmulatorManager();
  let hasRomBeenLoaded = false;

  let options = $state(loadOptions());
  $effect(() => {
    manager.updateOptions(options);
    saveOptions(options);
  });

  let input = new InputManager();
  input.onPause(() => {
    if (!hasRomBeenLoaded) {
      return;
    }
    manager.toggle_execution();
    if (!manager.running) {
      currentPage = 0;
    }
  });
  input.onControlInput((input, pressed) => {
    manager.updateInput(input, pressed);
  });
  input.onKeybindPressed((keybind) => {
    console.log(keybind);
    switch (keybind) {
      case "Zoom in":
        options.scaleOffset = Math.min(options.scaleOffset + 1, 5);
        break;
      case "Zoom out":
        options.scaleOffset = Math.max(options.scaleOffset - 1, -5);
        break;
    }
  });

  let currentPage = $state(0);
  let transitionDuration = 300;
  let transitionLength = 200;
  let lastPage = 0;
  function getTransition() {
    if (!options.uiTransitions) {
      return { y: 0, duration: 0 };
    }
    let sign = currentPage > lastPage ? 1 : -1;
    lastPage = currentPage;
    return { y: sign * transitionLength, duration: transitionDuration };
  }

  let popupVisible = $state(false);
  let popupText = $state("");
  function showMessage(msg: string) {
    popupText = msg;
    popupVisible = true;
    setTimeout(() => {
      popupVisible = false;
    }, 2000);
  }

  manager.onRomLoaded((success, info) => {
    if (success) {
      document.title = `${info} - DMG-2025`;
      console.info("Successfully loaded ROM");
      if (!hasRomBeenLoaded) {
        showMessage("Press Esc to return to menu");
        hasRomBeenLoaded = true;
      }
      manager.toggle_execution();
    } else {
      let msg = `Failed to load ROM: ${info}`;
      console.error(msg);
      showMessage(msg);
    }
  });

  onMount(async () => manager.initialize(options));
</script>

<svelte:window
  on:keydown={(event) => input.handleKey(event, true)}
  on:keyup={(event) => input.handleKey(event, false)}
/>

<main>
  {#if popupVisible}
    <p
      class="popup"
      in:fly={{ y: 200, duration: 600 }}
      out:fade={{ duration: 2000 }}
    >
      {popupText}
    </p>
  {/if}
  <canvas id="canvas" tabindex="-1"></canvas>
  {#if !manager.running}
    <div
      class="menu"
      transition:fade={{ duration: options.uiTransitions ? 100 : 0 }}
    >
      <div class="menu-sidebar">
        <button onclick={() => (currentPage = 0)}>MAIN</button>
        <button onclick={() => (currentPage = 1)}>BROWSER</button>
        <button onclick={() => (currentPage = 2)}>VISUALS</button>
        <button onclick={() => (currentPage = 3)}>INPUT</button>
      </div>
      {#if currentPage == 1}
        <div class="menu-container" in:fly={getTransition()}>
          <BrowserPage {manager} />
        </div>
      {:else if currentPage == 2}
        <div class="menu-container" in:fly={getTransition()}>
          <VisualsPage bind:options></VisualsPage>
        </div>
      {:else if currentPage == 3}
        <div class="menu-container" in:fly={getTransition()}>
          <InputPage {input} />
        </div>
      {:else}
        <div class="menu-container" in:fly={getTransition()}>
          <MainPage {manager} bind:options onBrowse={() => (currentPage = 1)} />
        </div>
      {/if}
    </div>
  {/if}
</main>
