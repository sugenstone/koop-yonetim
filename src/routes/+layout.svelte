<script lang="ts">
  import type { LayoutProps } from './$types';
  import modeobserver from './utils/modeobserver';
  import { onMount } from 'svelte';

  let { children }: LayoutProps = $props();

  onMount(modeobserver);
</script>

{@render children()}
