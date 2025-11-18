<script lang="ts">
  import MainPage from "./MainPage.svelte";
  import BrowserPage from "./BrowserPage.svelte";
  import VisualsPage from "./VisualsPage.svelte";
  import InputPage from "./InputPage.svelte";

  import InputManager from "./input.svelte";
  import EmulatorManager from "./manager.svelte";
  import Database from "./db.svelte";

  import { fade, fly } from "svelte/transition";
  import { loadOptions, saveOptions } from "./options.svelte";
  import { onMount, tick } from "svelte";

  let db = new Database();
  let manager = new EmulatorManager();
  let romHash = 0;

  let hasRomBeenLoaded = false;
  let saveDisabled = $state(true);
  let loadDisabled = $state(true);
  let saveSlot = $state(1);
  const changeSaveSlot = async (change: number) => {
    saveSlot += change;
    if (saveSlot > 10) {
      saveSlot = 1;
    } else if (saveSlot < 1) {
      saveSlot = 10;
    }
    if (hasRomBeenLoaded) {
      loadDisabled = (await db.getState(romHash, saveSlot)) === null;
    }
    showPopupMessage(`Selected slot ${saveSlot}`);
  };
  const saveState = () => {
    manager.serializeCPU();
  };
  const loadState = async () => {
    let state = await db.getState(romHash, saveSlot);
    if (state) {
      manager.deserializeCPU(state);
    }
  };

  let options = $state(loadOptions());
  $effect(() => {
    manager.setSpeed(options.speed);
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
  input.onKeybindPressed((keybind, pressed) => {
    if (keybind.slice(0, 9) === "Save slot") {
      let slot = parseInt(keybind.slice(10));
      saveSlot = slot;
      showPopupMessage(`Selected slot ${saveSlot}`);
    }
    switch (keybind) {
      case "Zoom in":
        if (pressed) {
          options.scaleOffset = Math.min(options.scaleOffset + 1, 5);
        }
        break;
      case "Zoom out":
        if (pressed) {
          options.scaleOffset = Math.max(options.scaleOffset - 1, -5);
        }
        break;
      case "Fast forward":
        if (pressed) {
          manager.setSpeed(options.fast_forward_speed);
        } else {
          manager.setSpeed(options.speed);
        }
        break;
      case "Save state":
        if (pressed) {
          saveState();
        }
        break;
      case "Load state":
        if (pressed) {
          loadState();
        }
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
  function showPopupMessage(msg: string) {
    popupText = msg;
    popupVisible = false;
    tick();
    popupVisible = true;
    setTimeout(() => {
      popupVisible = false;
    }, 400);
  }

  manager.onRomLoaded(async (title, hash) => {
    romHash = hash;
    saveSlot = 1;
    saveDisabled = false;
    loadDisabled = (await db.getState(romHash, saveSlot)) === null;
    document.title = `${title} - DMG-2025`;
    console.info(`Loaded ROM "${title}" with hash ${hash}`);
    if (!hasRomBeenLoaded) {
      showPopupMessage("Press Esc to return to menu");
      hasRomBeenLoaded = true;
    }
    manager.toggle_execution();
  });
  manager.onCPUSerialization((result) => {
    db.saveState(romHash, saveSlot, result);
    loadDisabled = false;
    console.info(`Serialized state with length of ${result.length}`);
    showPopupMessage(`Saved state to slot ${saveSlot}`);
  });
  manager.onCPUDeserialization(() => {
    console.info(`Deserialized state`);
    if (!manager.running) {
      manager.toggle_execution();
    }
    showPopupMessage(`Loaded state from slot ${saveSlot}`);
  });
  manager.onError((error) => {
    console.error(error);
    showPopupMessage(error);
  });

  onMount(() => manager.initialize(options));
</script>

<svelte:window
  on:keydown={(event) => input.handleKey(event, true)}
  on:keyup={(event) => input.handleKey(event, false)}
/>

<main>
  {#if popupVisible}
    <p
      class="popup"
      in:fly={{ y: 200, duration: 400 }}
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
          <MainPage
            {manager}
            bind:options
            onBrowse={() => (currentPage = 1)}
            onSave={saveState}
            onLoad={loadState}
            onSaveSlotChange={changeSaveSlot}
            {saveDisabled}
            {loadDisabled}
            {saveSlot}
          />
        </div>
      {/if}
    </div>
  {/if}
</main>
