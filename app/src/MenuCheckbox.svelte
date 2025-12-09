<script lang="ts">
  import checkedIconUrl from "../assets/checkboxChecked.png";
  import uncheckedIconUrl from "../assets/checkbox.png";
  let {
    value = $bindable(),
    label = "",
    featured = false,
  }: { value: boolean; label?: string; featured?: boolean } = $props();
</script>

<button
  class={"img-button " + (featured ? "featured" : "")}
  onclick={() => (value = !value)}
>
  <img
    src={value ? checkedIconUrl : uncheckedIconUrl}
    loading="eager"
    alt="Checkbox"
  />
  <p>{label}</p>
</button>
