<script lang="ts">
  import InputManager from "./input.svelte";
  let { input }: { input: InputManager } = $props();
</script>

{#snippet inputList(mappings: Record<string, string>)}
  {#each Object.keys(mappings) as name (name)}
    <p>{name}</p>
    {#if input.mappingToRebind == name}
      <button style="color:grey">[ Rebinding... ]</button>
    {:else}
      <button onclick={() => (input.mappingToRebind = name)}
        >{mappings[name]}</button
      >
    {/if}
  {/each}
{/snippet}

<div class="input-container">
  <div class="menu-grid input-grid" tabindex="-1">
    <h3>Controls</h3>
    {@render inputList(input.controls)}
  </div>
  <div class="menu-grid input-grid" tabindex="-1">
    <h3>Keybinds</h3>
    {@render inputList(input.keybinds)}
  </div>
  <p class="break"></p>
  <button class="danger-button" onclick={() => input.setToDefaults()}
    >Set to defaults
  </button>
</div>
