<script lang="ts">
  import '../app.css';
  import type { LayoutProps } from './$types';
  import modeobserver from './utils/modeobserver';
  import { onMount } from 'svelte';
  import { Toaster } from 'svelte-sonner';

  let { children }: LayoutProps = $props();

  onMount(modeobserver);
</script>

<Toaster richColors position="top-right" closeButton />
{@render children()}
