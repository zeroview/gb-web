<script lang="ts">
  import { run } from "DMG-2025";

  let running = $state(false);

  let files: FileList | undefined = $state();
  $effect(() => {
    // Open selected file as byte array
    if (files) {
      files[0].arrayBuffer().then(run_emulator);
    }
  });

  function run_emulator(rom: ArrayBuffer) {
    running = true;

    // Initialize WASM emulator with ROM byte array
    let proxy = run(new Uint8Array(rom));
    proxy.test("hello from JS");

    // Progress emulator every animation frame for the duration it took to make last frame
    let lastTime = performance.now();
    function frame() {
      let currentTime = performance.now();
      let millis = Math.min(100, Math.max(0, currentTime - lastTime));
      console.log(millis);
      lastTime = currentTime;

      proxy.run_cpu(millis);
      window.requestAnimationFrame(frame);
    }
    window.requestAnimationFrame(frame);
  }
</script>

<main>
  <canvas id="canvas"></canvas>
  {#if !running}
    <div class="menu">
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
