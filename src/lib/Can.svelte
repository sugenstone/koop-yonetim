<!--
  Can.svelte — CASL benzeri izin kapisi.

  Kullanim:
    <Can permission="kasa.olustur">
      <Button onclick={yeni}>Yeni Kasa</Button>
    </Can>

    <Can permission="kasa.duzenle" mode="disable">
      <Button onclick={kaydet}>Kaydet</Button>
    </Can>

    <Can any={['hisse.yonet', 'hisse.satis']}>
      ...
    </Can>

  mode:
    - "hide" (varsayilan) — izin yoksa hic render etme
    - "disable" — izin yoksa tiklanamaz hale getir (pointer-events:none + opacity)
-->
<script lang="ts">
	import { myPermissions } from '$lib/permissions';
	import { getCurrentUser } from '$lib/api-client';
	import type { Snippet } from 'svelte';

	type Props = {
		permission?: string;
		any?: string[];
		all?: string[];
		mode?: 'hide' | 'disable';
		children: Snippet;
		fallback?: Snippet;
	};

	let { permission, any, all, mode = 'hide', children, fallback }: Props = $props();

	const user = $derived(getCurrentUser());
	const izinli = $derived.by(() => {
		if (!user) return false;
		if (user.rol === 'admin') return true;
		const set = $myPermissions;
		if (permission && !set.has(permission)) return false;
		if (any && !any.some((k) => set.has(k))) return false;
		if (all && !all.every((k) => set.has(k))) return false;
		return true;
	});
</script>

{#if izinli}
	{@render children()}
{:else if mode === 'disable'}
	<span
		class="pointer-events-none inline-block cursor-not-allowed opacity-40"
		title="Yetkiniz yok"
		aria-disabled="true"
	>
		{@render children()}
	</span>
{:else if fallback}
	{@render fallback()}
{/if}
