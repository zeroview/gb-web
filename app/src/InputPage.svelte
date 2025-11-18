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

<div
  class="menu-grid"
  style="grid-template-columns: 15rem 15rem; overflow-y: scroll;"
>
  <h3>Controls</h3>
  {@render inputList(input.controls)}
  <h3>Keybinds</h3>
  {@render inputList(input.keybinds)}
  <button class="danger-button" onclick={() => input.setToDefaults()}
    >Set to defaults</button
  >
</div>
