<script lang="ts">
  import { run } from "DMG-2025";

  let started = false;

  function run_emulator() {
    started = true;
    run();
  }
</script>

<main>
  <canvas id="canvas"></canvas>
  {#if !started}
    <button on:click={run_emulator}> Run emulator </button>
  {/if}
</main>
