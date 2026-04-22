<script lang="ts">
  interface Item {
    value: number | string;
    name: string;
  }

  interface Props {
    value: number | string | '';
    items: Item[];
    placeholder?: string;
    id?: string;
    disabled?: boolean;
  }

  let { value = $bindable(''), items, placeholder = 'Seçin...', id, disabled = false }: Props = $props();

  let query = $state('');
  let open = $state(false);
  let containerEl: HTMLDivElement;

  // Seçili öğenin adını göster
  let selectedName = $derived(
    value !== '' ? (items.find((i) => i.value === value)?.name ?? '') : ''
  );

  // Filtreli liste
  let filtered = $derived(
    query.trim() === ''
      ? items
      : items.filter((i) => i.name.toLowerCase().includes(query.toLowerCase()))
  );

  function openDropdown() {
    if (disabled) return;
    query = '';
    open = true;
  }

  function select(item: Item) {
    value = item.value;
    query = '';
    open = false;
  }

  function clear() {
    value = '';
    query = '';
    open = false;
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
    } else if (e.key === 'Enter' && filtered.length === 1) {
      select(filtered[0]);
    }
  }

  function onClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }
</script>

<svelte:window onclick={onClickOutside} />

<div bind:this={containerEl} class="relative" {id}>
  {#if open}
    <!-- Arama input -->
    <div class="relative">
      <input
        type="text"
        bind:value={query}
        onkeydown={onInputKeydown}
        placeholder="İsim ile ara..."
        autofocus
        class="block w-full rounded-lg border border-gray-300 bg-white px-3 py-2 pr-8 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
      />
      <button
        type="button"
        onclick={() => (open = false)}
        class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        aria-label="Kapat"
      >
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Dropdown listesi -->
    <ul
      class="absolute z-50 mt-1 max-h-56 w-full overflow-y-auto rounded-lg border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-700"
    >
      <!-- Temizle seçeneği -->
      <li>
        <button
          type="button"
          onclick={clear}
          class="flex w-full items-center px-3 py-2 text-sm italic text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-600"
        >
          — {placeholder} —
        </button>
      </li>
      {#each filtered as item (item.value)}
        <li>
          <button
            type="button"
            onclick={() => select(item)}
            class="flex w-full items-center px-3 py-2 text-sm text-gray-800 hover:bg-primary-50 hover:text-primary-700 dark:text-gray-200 dark:hover:bg-primary-900/30 dark:hover:text-primary-300"
            class:bg-primary-50={value === item.value}
            class:text-primary-700={value === item.value}
            class:dark:bg-primary-900={value === item.value}
          >
            {item.name}
          </button>
        </li>
      {:else}
        <li class="px-3 py-2 text-sm text-gray-400 dark:text-gray-500">Sonuç bulunamadı</li>
      {/each}
    </ul>
  {:else}
    <!-- Kapalı hali — seçimi gösteren buton -->
    <button
      type="button"
      onclick={openDropdown}
      {disabled}
      class="flex w-full items-center justify-between rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-700"
    >
      <span class={value !== '' ? 'text-gray-900 dark:text-white' : 'text-gray-400 dark:text-gray-400'}>
        {value !== '' ? selectedName : `— ${placeholder} —`}
      </span>
      <svg class="h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-4.35-4.35M17 11A6 6 0 1 1 5 11a6 6 0 0 1 12 0z" />
      </svg>
    </button>
  {/if}
</div>
