<script lang="ts">
  let { value = $bindable(), label }: { value: boolean; label: string } =
    $props();
</script>

<button class="checkbox" onclick={() => (value = !value)}>
  <!-- Checkbox icon -->
  <svg
    version="1.1"
    xmlns="http://www.w3.org/2000/svg"
    preserveAspectRatio="xMinYMin meet"
    viewBox="0 0 9 12"
  >
    <rect x="1" y="2" width="7" height="1" />
    <rect x="0" y="3" width="1" height="7" />
    <rect x="8" y="3" width="1" height="7" />
    <rect x="1" y="10" width="7" height="1" />
    <!-- Don't render inner X when not checked -->
    {#if value}
      <rect x="2" y="4" width="1" height="1" />
      <rect x="6" y="4" width="1" height="1" />
      <rect x="3" y="5" width="1" height="1" />
      <rect x="5" y="5" width="1" height="1" />
      <rect x="4" y="6" width="1" height="1" />
      <rect x="3" y="7" width="1" height="1" />
      <rect x="5" y="7" width="1" height="1" />
      <rect x="2" y="8" width="1" height="1" />
      <rect x="6" y="8" width="1" height="1" />
    {/if}
  </svg>
  <p>{label}</p>
</button>
