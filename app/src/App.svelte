<script lang="ts" module>
  export type LoadedROMInfo = {
    hash: number;
    saveRAM: boolean;
    name: string;
  };
</script>

<script lang="ts">
  import logoUrl from "../assets/logo.png";
  import iconUrl from "../assets/icon.png";
  import playIconUrl from "../assets/play.png";
  import browseIconUrl from "../assets/browse.png";
  import inputIconUrl from "../assets/input.png";
  import optionsIconUrl from "../assets/options.png";
  import pauseIconUrl from "../assets/pause.png";
  import fastForwardIconUrl from "../assets/fastforward.png";
  import loadingAnimationUrl from "../assets/loading.gif";

  import MainPage from "./MainPage.svelte";
  import BrowserPage from "./BrowserPage.svelte";
  import OptionsPage from "./OptionsPage.svelte";
  import InputPage from "./InputPage.svelte";

  import InputManager from "./input.svelte";
  import EmulatorBridge from "./bridge.svelte";
  import Database from "./db.svelte";

  import { fade, fly } from "svelte/transition";
  import {
    loadOptions,
    OnscreenControlsOption,
    saveOptions,
  } from "./options.svelte";
  import { onMount, tick } from "svelte";

  // Handler initialization
  let db = new Database();
  let bridge = new EmulatorBridge();

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
    showPopup(error, 3000);
  };
  /// Shows a popup for an info message
  const showInfoPopup = (msg: string) => {
    popupColor = infoColor;
    showPopup(msg, 2000);
  };

  /// Info about loaded ROM
  let loadedROMInfo: LoadedROMInfo = $state({
    /// The hash of the contents of the currently running ROM
    hash: 0,
    /// If RAM should be externally saved for currently loaded ROM
    saveRAM: false,
    /// The name of the rom (file name or from browser)
    name: "",
  });

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
      db.getState(loadedROMInfo.hash, stateSlot)
        .then(() => (loadStateDisabled = false))
        .catch(() => (loadStateDisabled = true));
    }
    showInfoPopup(`Selected state slot ${stateSlot}`);
  };
  const saveState = async () => {
    try {
      // Serialize emulator state and save it to database
      let buffer = await bridge.serializeCPU();
      await db.saveState(loadedROMInfo.hash, stateSlot, buffer);
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
      let buffer = await db.getState(loadedROMInfo.hash, stateSlot);
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

  const saveRAM = async () => {
    if (!loadedROMInfo.saveRAM) {
      return;
    }
    let ram = await bridge.saveRAM();
    await db.saveRAM(loadedROMInfo.hash, ram);
  };

  const loadSavedRAM = async () => {
    // Check if RAM is saved
    if (loadedROMInfo.saveRAM) {
      // Get from database
      let ram = await db.getRAM(loadedROMInfo.hash).catch(console.warn);
      // Load into emulator if successful
      if (ram) {
        bridge.loadRAM(ram);
      }
    }
  };
  const loadROM = async (rom: ArrayBuffer, name: string, isZip: boolean) => {
    // Try to load ROM, if fails, show popup for reason
    let info = await bridge.loadROM(rom, isZip).catch(showErrorPopup);
    if (!info) {
      return;
    }

    loadedROMInfo = {
      hash: info.hash,
      saveRAM: info.should_be_saved,
      name,
    };
    stateSlot = 1;
    // Enable state loading if state exists for this slot
    db.getState(loadedROMInfo.hash, stateSlot)
      .then(() => (loadStateDisabled = false))
      .catch(() => (loadStateDisabled = true));

    // Load saved RAM into emulator
    loadSavedRAM();

    document.title = `${info.title} - DMG-2025`;
    console.info(
      `Loaded ROM file "${name}". Header: "${info.title}" Hash: ${info.hash}`,
    );
    if (!hasRomBeenLoaded) {
      if (!bridge.showOnscreenControls) {
        showPopup("Check Input page for controls", 6000);
      }
      hasRomBeenLoaded = true;
    }
    // Start emulation
    bridge.toggle_execution();
    setTimeout(() => {
      currentPage = 0;
    }, 100);
  };

  const reload = async () => {
    try {
      // Reload ROM
      await bridge.reload();
      // Set RAM if saved
      await loadSavedRAM();
      console.info("Reloaded ROM");
      bridge.toggle_execution();
    } catch (e) {
      showErrorPopup(e as string);
    }
  };

  /// Global options for emulator, saved automatically into LocalStorage
  let options = $state(loadOptions());
  $effect(() => {
    // Update options on state change
    bridge.setSpeed(options.speed);
    if (options.onScreenControls != OnscreenControlsOption.Auto) {
      bridge.showOnscreenControls =
        options.onScreenControls == OnscreenControlsOption.Visible;
    }
    bridge.updateOptions(options);
    saveOptions(options);
  });

  const pause = () => {
    if (!hasRomBeenLoaded) {
      return;
    }
    bridge.toggle_execution();
    // RAM is automatically saved on pause (if enabled for ROM)
    if (!bridge.running) {
      saveRAM();
    }
  };

  /// Input manager saves keybinds and calls callbacks on input
  let input = new InputManager();
  input.onPause(pause);
  input.onControlInput((input, pressed) => {
    bridge.updateInput(input, pressed);
  });
  input.onKeybindPressed((keybind, pressed) => {
    // Handle setting state slots 1-10
    if (keybind.slice(0, 10) === "State slot" && pressed) {
      let slot = parseInt(keybind.slice(11));
      stateSlot = slot;
      showInfoPopup(`Selected state slot ${stateSlot}`);
    }
    switch (keybind) {
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
  input.onKeyboardFocusEnd(() => {
    // Remove focus from element that has taken keyboard focus
    (document.activeElement as HTMLElement)?.blur?.();
  });

  let filters = $state({
    search: "",
    featured: true,
    games: false,
    demos: false,
    tools: false,
    music: false,
  });

  // Constants for transitions
  let transitionDuration = 300;
  let transitionLength = 200;

  // State variables for UI
  let currentPage = $state(0);
  let lastPage = 0;

  const getUseLogoIcon = () => {
    return window.matchMedia("(max-width: 850px)").matches;
  };
  let useLogoIcon = $state(getUseLogoIcon());
  const getUseSidebarIcons = () => {
    return window.matchMedia("(max-width: 720px").matches;
  };
  let useSidebarIcons = $state(getUseSidebarIcons());

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

  onMount(async () => {
    showInfoPopup("Downloading emulator...");
    await bridge.initialize(options);
    showInfoPopup("Emulator initialized!");
  });
</script>

<svelte:window
  on:resize={() => {
    useLogoIcon = getUseLogoIcon();
    useSidebarIcons = getUseSidebarIcons();
  }}
  on:beforeunload={(event) => {
    // If the RAM should be saved on this ROM,
    // notify on tab close about possibility of losing save data
    if (!loadedROMInfo.saveRAM || !bridge.running) {
      return;
    }
    event.preventDefault();
    event.returnValue = true;
    return "...";
  }}
/>

<svelte:document
  on:keydown={(event) => {
    if (
      options.onScreenControls == OnscreenControlsOption.Auto &&
      bridge.showOnscreenControls
    ) {
      bridge.showOnscreenControls = false;
      bridge.updateOptions(options);
    }
    input.handleKey(event, true);
  }}
  on:keyup={(event) => input.handleKey(event, false)}
  on:pointermove={(event) => {
    bridge.updatePointerPos(
      -1,
      event.clientX * window.devicePixelRatio,
      event.clientY * window.devicePixelRatio,
    );
  }}
  on:pointerdown={(event) => {
    bridge.updatePointerPressed(-1, true).then(() => {
      bridge.updatePointerPos(
        -1,
        event.clientX * window.devicePixelRatio,
        event.clientY * window.devicePixelRatio,
      );
    });
  }}
  on:pointerup={() => {
    bridge.updatePointerPressed(-1, false);
  }}
  on:touchmove={(event) => {
    for (let i = 0; i < event.changedTouches.length; i++) {
      let touch = event.changedTouches[i];
      bridge.updatePointerPos(
        touch.identifier,
        touch.clientX * window.devicePixelRatio,
        touch.clientY * window.devicePixelRatio,
      );
    }
  }}
  on:touchstart={(event) => {
    if (
      options.onScreenControls == OnscreenControlsOption.Auto &&
      !bridge.showOnscreenControls
    ) {
      bridge.showOnscreenControls = true;
      bridge.updateOptions(options);
    }
    for (let i = 0; i < event.changedTouches.length; i++) {
      let touch = event.changedTouches[i];
      bridge.updatePointerPressed(touch.identifier, true);
      bridge.updatePointerPos(
        touch.identifier,
        touch.clientX * window.devicePixelRatio,
        touch.clientY * window.devicePixelRatio,
      );
    }
  }}
  on:touchend={(event) => {
    for (let i = 0; i < event.changedTouches.length; i++) {
      let touch = event.changedTouches[i];
      bridge.updatePointerPressed(touch.identifier, false);
    }
  }}
  on:touchcancel={(event) => {
    for (let i = 0; i < event.changedTouches.length; i++) {
      let touch = event.changedTouches[i];
      bridge.updatePointerPressed(touch.identifier, false);
    }
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
  {#if !bridge.initialized}
    <img class="loading" src={loadingAnimationUrl} alt="Loading..." />
  {/if}
  <canvas id="canvas" tabindex="-1"></canvas>

  {#if !bridge.running}
    <div
      class="menu"
      transition:fade={{ duration: options.uiTransitions ? 100 : 0 }}
    >
      {#snippet menuButton(text: string, imgUrl: string, pageIndex: number)}
        <button
          onclick={() => (currentPage = pageIndex)}
          style="{currentPage === pageIndex
            ? 'text-decoration-color: #d1d1d1;'
            : ''}}"
          class="img-button"
        >
          <img src={imgUrl} alt={text} />
          {#if !useSidebarIcons}
            <p>{text}</p>
          {/if}
        </button>
      {/snippet}
      <div class="menu-sidebar">
        <a href="https://github.com/zeroview/DMG-2025" class="info-button">
          <img src={useLogoIcon ? iconUrl : logoUrl} alt="DMG-2025" />
          <p>v. 1.0.0</p>
        </a>
        <div class="menu-sidebar-buttons">
          {@render menuButton("PLAY", playIconUrl, 0)}
          {@render menuButton("BROWSER", browseIconUrl, 1)}
          {@render menuButton("INPUT", inputIconUrl, 2)}
          {@render menuButton("OPTIONS", optionsIconUrl, 3)}
        </div>
      </div>
      {#if currentPage == 1}
        <div class="menu-container" in:fly={getTransition()}>
          <BrowserPage
            bind:filters
            onLoadRom={loadROM}
            onKeyboardFocus={(focus) => (input.keyboardFocused = focus)}
          />
        </div>
      {:else if currentPage == 2}
        <div class="menu-container" in:fly={getTransition()}>
          <InputPage {input} />
        </div>
      {:else if currentPage == 3}
        <div class="menu-container" in:fly={getTransition()}>
          <OptionsPage
            bind:options
            {db}
            successCallback={showInfoPopup}
            errorCallback={showErrorPopup}
          />
        </div>
      {:else}
        <div class="menu-container main-menu" in:fly={getTransition()}>
          <MainPage
            bind:options
            info={loadedROMInfo}
            onBrowse={() => (currentPage = 1)}
            onLoadRom={loadROM}
            onReload={reload}
            onResume={pause}
            onSaveState={saveState}
            onLoadState={loadState}
            onSaveSlotChange={changeSaveSlot}
            romLoaded={hasRomBeenLoaded}
            {loadStateDisabled}
            {stateSlot}
          />
        </div>
      {/if}
    </div>
  {:else}
    <button
      class="ui-button"
      style="left:0"
      onclick={pause}
      draggable="false"
      transition:fade={{ duration: options.uiTransitions ? 100 : 0 }}
    >
      <img src={pauseIconUrl} alt="Pause" draggable="false" />
    </button>
    <button
      class="ui-button"
      style="right:0"
      ontouchstart={() => bridge.setSpeed(options.fast_forward_speed)}
      ontouchend={() => bridge.setSpeed(options.speed)}
      onmousedown={() => bridge.setSpeed(options.fast_forward_speed)}
      onmouseup={() => bridge.setSpeed(options.speed)}
      transition:fade={{ duration: options.uiTransitions ? 100 : 0 }}
    >
      <img src={fastForwardIconUrl} alt="Fast-forward" draggable="false" />
    </button>
  {/if}
</main>
