<script lang="ts">
  import type { Ace } from "ace-builds";
  import ace from "ace-builds/src-noconflict/ace?client";
  import * as aceGithubTheme from "ace-builds/src-noconflict/theme-github?client";

  import { onMount } from "svelte";

  export let editSession: Ace.EditSession;

  let editorDiv: HTMLDivElement | undefined;
  let editor: Ace.Editor | undefined;

  onMount(() => {
    let renderer = new ace.VirtualRenderer(editorDiv);
    editor = new ace.Editor(renderer, editSession);
    editor.setTheme(aceGithubTheme);
  });

  $: if (editor !== undefined) {
    editor.setSession(editSession);
  }
</script>

<div bind:this={editorDiv} class="editor" />

<style>
  .editor {
    width: 100%;
    height: 100%;
  }
</style>
