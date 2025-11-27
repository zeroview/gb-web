<script lang="ts">
  let {
    value = $bindable(),
    label,
    featured,
  }: { value: boolean; label: string; featured?: boolean } = $props();
</script>

<button
  class={"img-button " + (featured ? "featured" : "")}
  onclick={() => (value = !value)}
>
  {#if value}
    <img src="/app/assets/checkboxChecked.png" alt="Checked checkbox" />
  {:else}
    <img src="/app/assets/checkbox.png" alt="Checkbox" />
  {/if}
  <p>{label}</p>
</button>
