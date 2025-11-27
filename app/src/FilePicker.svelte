<script lang="ts">
  let {
    children,
    cssClass,
    fileTypes,
    onPick,
  }: {
    children: any | undefined;
    cssClass?: string;
    fileTypes: string;
    onPick: (file: File) => void;
  } = $props();

  let fileInput: HTMLInputElement | undefined = $state();
  let files: FileList | undefined = $state();
  $effect(() => {
    if (files) {
      onPick(files[0]);
    }
  });
</script>

<button
  class={cssClass}
  onclick={() => {
    fileInput?.click();
  }}
>
  <input
    bind:this={fileInput}
    accept={fileTypes}
    type="file"
    bind:files
    style="display: none"
  />
  {@render children?.()}
</button>
