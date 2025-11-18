<script lang="ts">
  let {
    value = $bindable(),
    min,
    max,
    step,
    valueLabelCallback,
  }: {
    value: number;
    min: number;
    max: number;
    step: number;
    valueLabelCallback?: (value: number) => string;
  } = $props();

  let valueLabel = $derived.by(() => {
    if (valueLabelCallback === undefined) {
      return value.toString();
    } else {
      return valueLabelCallback(value);
    }
  });
</script>

<div class="slider-row">
  <input type="range" bind:value {min} {max} {step} style="width: 250px" />
  <p style="width: 5rem;">{valueLabel}</p>
</div>
