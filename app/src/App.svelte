<script lang="ts">
  import MainPage from "./MainPage.svelte";
  import BrowserPage from "./BrowserPage.svelte";
  import VisualsPage from "./VisualsPage.svelte";
  import InputPage from "./InputPage.svelte";

  import InputManager from "./input.svelte";
  import EmulatorBridge from "./bridge.svelte";
  import Database from "./db.svelte";

  import { fade, fly } from "svelte/transition";
  import { loadOptions, saveOptions } from "./options.svelte";
  import { onMount, tick } from "svelte";

  // Handler initialization
  let db = new Database();
  let bridge = new EmulatorBridge();
  /// The hash of the contents of the currently running ROM
  let romHash = 0;

  // State for info / error popup
  let infoColor = "#ffffff";
  let errorColor = "#ff0000";
  let popupColor = $state(infoColor);
  let popupVisible = $state(false);
  let popupText = $state("");

  const showPopup = (text: string, length: number) => {
    popupText = text;
    // Restart animation by calling tick()
    // and re-rendering component in between visiblity change
    popupVisible = false;
    tick();
    popupVisible = true;
    setTimeout(() => {
      popupVisible = false;
    }, length);
  };
  /// Shows a popup for an error message
  const showErrorPopup = (error: string) => {
    console.error(error);
    popupColor = errorColor;
    showPopup(error, 2000);
  };
  /// Shows a popup for an info message
  const showInfoPopup = (msg: string) => {
    popupColor = infoColor;
    showPopup(msg, 500);
  };

  /// If ROM has been loaded in this session
  let hasRomBeenLoaded = $state(false);
  /// If state can't be loaded from currently selected slot
  let loadStateDisabled = $state(true);
  /// Slot to save state to
  let stateSlot = $state(1);

  const changeSaveSlot = async (change: number) => {
    stateSlot += change;
    if (stateSlot > 10) {
      stateSlot = 1;
    } else if (stateSlot < 1) {
      stateSlot = 10;
    }
    // Check if state can be loaded from slot
    if (hasRomBeenLoaded) {
      db.getState(romHash, stateSlot)
        .then(() => (loadStateDisabled = false))
        .catch(() => (loadStateDisabled = true));
    }
    showInfoPopup(`Selected state slot ${stateSlot}`);
  };
  const saveState = async () => {
    try {
      // Serialize emulator state and save it to database
      let buffer = await bridge.serializeCPU();
      await db.saveState(romHash, stateSlot, buffer);
      loadStateDisabled = false;
      console.info(`Serialized state with length of ${buffer.length}`);
      showInfoPopup(`Saved state to slot ${stateSlot}`);
    } catch (error) {
      showErrorPopup(error as string);
    }
  };
  const loadState = async () => {
    try {
      // Get state from database and deserialize new emulator struct from it
      let buffer = await db.getState(romHash, stateSlot);
      await bridge.deserializeCPU(buffer);
      console.info(`Deserialized state`);
      if (!bridge.running) {
        bridge.toggle_execution();
      }
      showInfoPopup(`Loaded state from slot ${stateSlot}`);
    } catch (error) {
      showErrorPopup(error as string);
    }
  };

  /// If RAM should be externally saved for currently loaded ROM
  let ramShouldBeSaved = false;
  const saveRAM = async () => {
    if (!ramShouldBeSaved) {
      return;
    }
    let ram = await bridge.saveRAM();
    await db.saveRAM(romHash, ram);
  };

  const loadROM = async (rom: ArrayBuffer, isZip: boolean) => {
    // Try to load ROM, if fails, show popup for reason
    let info = await bridge.loadROM(rom, isZip).catch(showErrorPopup);
    if (!info) {
      return;
    }

    romHash = info.hash;
    ramShouldBeSaved = info.should_be_saved;
    stateSlot = 1;
    // Enable state loading if state exists for this slot
    db.getState(romHash, stateSlot)
      .then(() => (loadStateDisabled = false))
      .catch(() => (loadStateDisabled = true));

    // Load saved RAM into emulator
    if (ramShouldBeSaved) {
      let ram = await db.getRAM(romHash).catch(console.warn);
      if (ram) {
        bridge.loadRAM(ram);
      }
    }

    document.title = `${info.title} - DMG-2025`;
    console.info(`Loaded ROM "${info.title}" with hash ${info.hash}`);
    if (!hasRomBeenLoaded) {
      showInfoPopup("Press Esc to return to menu");
      hasRomBeenLoaded = true;
    }
    // Start emulation
    bridge.toggle_execution();
  };

  /// Global options for emulator, saved automatically into LocalStorage
  let options = $state(loadOptions());
  $effect(() => {
    bridge.setSpeed(options.speed);
    bridge.updateOptions(options);
    saveOptions(options);
  });

  /// Input manager saves keybinds and calls callbacks on input
  let input = new InputManager();
  input.onPause(() => {
    if (!hasRomBeenLoaded) {
      return;
    }
    bridge.toggle_execution();
    if (!bridge.running) {
      currentPage = 0;
      // RAM is automatically saved on pause (if enabled for ROM)
      saveRAM();
    }
  });
  input.onControlInput((input, pressed) => {
    bridge.updateInput(input, pressed);
  });
  input.onKeybindPressed((keybind, pressed) => {
    // Handle setting state slots 1-10
    if (keybind.slice(0, 10) === "State slot") {
      let slot = parseInt(keybind.slice(11));
      stateSlot = slot;
      showInfoPopup(`Selected state slot ${stateSlot}`);
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
          bridge.setSpeed(options.fast_forward_speed);
        } else {
          bridge.setSpeed(options.speed);
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

  // Constants for transitions
  let transitionDuration = 300;
  let transitionLength = 200;

  // State variables for UI
  let currentPage = $state(0);
  let lastPage = 0;

  const getTransition = () => {
    if (!options.uiTransitions) {
      // Disable transitions if user wishes
      return { y: 0, duration: 0 };
    }
    // Flip transition length based on last page
    // This makes it so transition mimics scrolling
    // (for example, when going from first page in sidebar to the second
    //  the new page is animated from the bottom)
    let sign = currentPage > lastPage ? 1 : -1;
    lastPage = currentPage;
    return { y: sign * transitionLength, duration: transitionDuration };
  };

  onMount(() => bridge.initialize(options));
</script>

<svelte:window
  on:keydown={(event) => input.handleKey(event, true)}
  on:keyup={(event) => input.handleKey(event, false)}
  on:beforeunload={(event) => {
    // If the RAM should be saved on this ROM,
    // notify on tab close about possibility of losing save data
    if (!ramShouldBeSaved || !bridge.running) {
      return;
    }
    event.preventDefault();
    event.returnValue = true;
    return "...";
  }}
/>

<main>
  {#if popupVisible}
    <p
      class="popup"
      style={`color: ${popupColor}`}
      in:fly={{ y: 200, duration: 400 }}
      out:fade={{ duration: 2000 }}
    >
      {popupText}
    </p>
  {/if}
  <canvas id="canvas" tabindex="-1"></canvas>
  {#if !bridge.running}
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
          <BrowserPage onLoadRom={loadROM} />
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
            bind:options
            onBrowse={() => (currentPage = 1)}
            onLoadRom={loadROM}
            onSaveState={saveState}
            onLoadState={loadState}
            onSaveSlotChange={changeSaveSlot}
            saveStateDisabled={!hasRomBeenLoaded}
            {loadStateDisabled}
            {stateSlot}
          />
        </div>
      {/if}
    </div>
  {/if}
</main>
