<script lang="ts">
  let {
    value = $bindable(),
    min = 0,
    max = 100,
    step = 1,
    values,
    labelFormatter,
  }: {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    values?: number[];
    labelFormatter?: (value: number) => string;
  } = $props();

  // svelte-ignore non_reactive_update
  let getter = () => value;
  // svelte-ignore non_reactive_update
  let setter = (val: number) => (value = val);

  // If provided a list of numbers, use slider value to index through those
  if (values !== undefined) {
    min = 0;
    max = values.length - 1;
    step = 1;
    getter = () => values.indexOf(value);
    setter = (index) => (value = values[index]);
  }

  let valueLabel = $derived.by(() => {
    if (labelFormatter === undefined) {
      return value.toString();
    } else {
      return labelFormatter(value);
    }
  });
</script>

<div class="slider-row">
  <input
    type="range"
    bind:value={getter, setter}
    {min}
    {max}
    {step}
    style="width: 250px"
  />
  <p style="width: 5rem;">{valueLabel}</p>
</div>
