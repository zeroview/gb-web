<script lang="ts">
  import { spawn_event_loop, Proxy } from "DMG-2025";
  import { fade } from "svelte/transition";

  const inputMap: Record<string, string> = {
    Right: "ArrowRight",
    Left: "ArrowLeft",
    Up: "ArrowUp",
    Down: "ArrowDown",
    A: "x",
    B: "z",
    Select: "Backspace",
    Start: "Enter",
  };

  let running = $state(false);
  let files: FileList | undefined = $state();
  $effect(() => {
    // Open selected file as byte array
    if (files) {
      files[0].arrayBuffer().then(load_rom);
    }
  });

  let proxy: Proxy | undefined = undefined;

  function resume() {
    // Progress emulator every animation frame for the duration it took to make last frame
    let lastTime = performance.now();
    function frame() {
      if (!running) {
        return;
      }
      let currentTime = performance.now();
      let millis = Math.min(17, Math.max(0, currentTime - lastTime));
      console.log(millis);
      lastTime = currentTime;

      proxy?.run_cpu(millis);
      window.requestAnimationFrame(frame);
    }
    window.requestAnimationFrame(frame);
  }

  function load_rom(rom: ArrayBuffer) {
    if (!proxy) {
      proxy = spawn_event_loop();
    }
    running = true;
    proxy.load_rom(new Uint8Array(rom));
    resume();
  }

  function handleKey(event: KeyboardEvent, pressed: boolean) {
    if (pressed && event.key == "Escape") {
      if (!proxy) {
        return;
      }
      running = !running;
      if (running) {
        resume();
      }
    }
    for (let key of Object.keys(inputMap)) {
      if (inputMap[key] === event.key) {
        console.log(event.key);
        proxy?.update_input(key, pressed);
      }
    }
  }
</script>

<svelte:window
  on:keydown={(event) => handleKey(event, true)}
  on:keyup={(event) => handleKey(event, false)}
/>

<main>
  <canvas id="canvas"></canvas>
  {#if !running}
    <div class="menu" transition:fade={{ duration: 100 }}>
      <input
        id="fileInput"
        accept=".gb"
        type="file"
        bind:files
        style="display: none"
      />
      <button onclick={() => document.getElementById("fileInput")?.click()}>
        Load ROM
      </button>
    </div>
  {/if}
</main>
