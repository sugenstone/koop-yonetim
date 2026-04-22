<script lang="ts" generics="T">
  import {
    createTable,
    getCoreRowModel,
    getSortedRowModel,
    getFilteredRowModel,
    getPaginationRowModel,
    type SortingState,
    type VisibilityState,
    type PaginationState,
    type TableOptions
  } from '@tanstack/table-core';
  import {
    Input,
    Button,
    Dropdown,
    DropdownItem,
    Checkbox,
    Table,
    TableHead,
    TableHeadCell,
    TableBody,
    Select
  } from 'flowbite-svelte';
  import {
    SearchOutline,
    ChevronUpOutline,
    ChevronDownOutline,
    ChevronSortOutline,
    FileExportOutline,
    AdjustmentsHorizontalOutline,
    ChevronLeftOutline,
    ChevronRightOutline
  } from 'flowbite-svelte-icons';
  import { untrack } from 'svelte';
  import type { Snippet } from 'svelte';
  import {
    toTanstackColumns,
    rowsToCsv,
    downloadCsv,
    type DataTableColumn
  } from './dataTableUtils';

  interface Props {
    data: T[];
    columns: DataTableColumn<T>[];
    /** Satır render snippet. (item, index, visibleCols) — visibleCols ile hücreleri koşullu render edin. */
    row: Snippet<[T, number, Set<string>]>;
    /** Opsiyonel empty state snippet */
    empty?: Snippet;
    /** Opsiyonel toolbar sağına ek buton vb. için snippet */
    toolbarExtra?: Snippet;
    searchable?: boolean;
    paginated?: boolean;
    pageSize?: number;
    pageSizes?: number[];
    exportable?: boolean;
    exportFileName?: string;
    columnToggle?: boolean;
    emptyMessage?: string;
    searchPlaceholder?: string;
    striped?: boolean;
    hoverable?: boolean;
    shadow?: boolean;
    /** Bind ile dışarıya: filtrelenmiş+sıralanmış tüm satırlar (sayfalama öncesi) */
    exportRows?: T[];
    /** Bind ile dışarıya: şu an görünür kolonlar */
    exportVisibleCols?: DataTableColumn<T>[];
  }

  let {
    data,
    columns,
    row,
    empty,
    toolbarExtra,
    searchable = true,
    paginated = true,
    pageSize = 25,
    pageSizes = [10, 25, 50, 100],
    exportable = true,
    exportFileName = 'veri',
    columnToggle = true,
    emptyMessage = 'Kayıt bulunamadı',
    searchPlaceholder = 'Ara...',
    striped = true,
    hoverable = true,
    shadow = true,
    exportRows = $bindable<T[]>([]),
    exportVisibleCols = $bindable<DataTableColumn<T>[]>([])
  }: Props = $props();

  // ─── State ──────────────────────────────────────────────────────────────
  let sorting = $state<SortingState>([]);
  let globalFilter = $state('');
  let columnVisibility = $state<VisibilityState>(
    Object.fromEntries(columns.filter((c) => c.hiddenByDefault).map((c) => [c.id, false]))
  );
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize });

  // ─── Tanstack Table ─────────────────────────────────────────────────────
  const tsColumns = $derived(toTanstackColumns(columns));

  const options = $derived<TableOptions<T>>({
    data,
    columns: tsColumns,
    state: {
      sorting,
      globalFilter,
      columnVisibility,
      pagination
    },
    onSortingChange: (u) => {
      sorting = typeof u === 'function' ? u(sorting) : u;
    },
    onGlobalFilterChange: (u) => {
      globalFilter = typeof u === 'function' ? u(globalFilter) : (u as string);
    },
    onColumnVisibilityChange: (u) => {
      columnVisibility = typeof u === 'function' ? u(columnVisibility) : u;
    },
    onPaginationChange: (u) => {
      pagination = typeof u === 'function' ? u(pagination) : u;
    },
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: paginated ? getPaginationRowModel() : undefined,
    enableSorting: true,
    enableGlobalFilter: searchable,
    globalFilterFn: 'includesString'
  });

  const table = $derived(createTable(options));

  // ─── Derived ────────────────────────────────────────────────────────────
  const visibleColumns = $derived(columns.filter((c) => columnVisibility[c.id] !== false));
  const visibleColIds = $derived(new Set(visibleColumns.map((c) => c.id)));
  const filteredRows = $derived(table.getFilteredRowModel().rows);
  const paginatedRows = $derived(
    paginated ? table.getPaginationRowModel().rows : filteredRows
  );
  const displayRows = $derived(paginatedRows.map((r) => r.original));
  const totalFiltered = $derived(filteredRows.length);
  const pageCount = $derived(table.getPageCount());
  const pageIndex = $derived(pagination.pageIndex);

  // ─── Sorting ────────────────────────────────────────────────────────────
  function toggleSort(colId: string): void {
    const col = columns.find((c) => c.id === colId);
    if (!col || col.sortable === false) return;
    const existing = sorting.find((s) => s.id === colId);
    if (!existing) {
      sorting = [{ id: colId, desc: false }];
    } else if (!existing.desc) {
      sorting = [{ id: colId, desc: true }];
    } else {
      sorting = [];
    }
  }

  function sortIcon(colId: string): 'asc' | 'desc' | 'none' {
    const s = sorting.find((x) => x.id === colId);
    if (!s) return 'none';
    return s.desc ? 'desc' : 'asc';
  }

  // ─── Pagination helpers ─────────────────────────────────────────────────
  function goFirst(): void {
    pagination = { ...pagination, pageIndex: 0 };
  }
  function goPrev(): void {
    pagination = { ...pagination, pageIndex: Math.max(0, pagination.pageIndex - 1) };
  }
  function goNext(): void {
    pagination = {
      ...pagination,
      pageIndex: Math.min(pageCount - 1, pagination.pageIndex + 1)
    };
  }
  function goLast(): void {
    pagination = { ...pagination, pageIndex: Math.max(0, pageCount - 1) };
  }
  function setPageSize(size: number): void {
    pagination = { pageIndex: 0, pageSize: size };
  }

  // ─── Export state sync ───────────────────────────────────────────────────
  $effect(() => {
    exportRows = filteredRows.map((r) => r.original);
    exportVisibleCols = [...visibleColumns];
  });

  // ─── Export ─────────────────────────────────────────────────────────────
  function exportCsv(): void {
    const visibleIds = new Set(visibleColumns.map((c) => c.id));
    // Export'ta sıralı + filtrelenmiş tüm veri
    const allFiltered = filteredRows.map((r) => r.original);
    const csv = rowsToCsv(allFiltered, columns, visibleIds);
    downloadCsv(csv, exportFileName);
  }

  // ─── Global search reset pagination ──────────────────────────────────────
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    globalFilter; // sadece globalFilter takip edilir
    untrack(() => {
      if (pagination.pageIndex !== 0) {
        pagination = { ...pagination, pageIndex: 0 };
      }
    });
  });
</script>

<div class="flex flex-col gap-3">
  <!-- ═══ Toolbar ═════════════════════════════════════════════════════════ -->
  {#if searchable || exportable || columnToggle || toolbarExtra}
    <div class="flex flex-wrap items-center gap-2">
      {#if searchable}
        <div class="relative flex-1 min-w-[200px] max-w-sm">
          <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
            <SearchOutline class="h-4 w-4 text-gray-400" />
          </div>
          <Input
            type="text"
            bind:value={globalFilter}
            placeholder={searchPlaceholder}
            class="pl-9"
          />
        </div>
      {/if}

      <div class="flex items-center gap-2 ml-auto">
        {#if toolbarExtra}
          {@render toolbarExtra()}
        {/if}

        {#if columnToggle}
          <Button color="alternative" size="sm" class="gap-1.5">
            <AdjustmentsHorizontalOutline class="h-4 w-4" />
            Kolonlar
          </Button>
          <Dropdown class="w-52 p-2 space-y-1">
            {#each columns as col (col.id)}
              <DropdownItem class="p-0">
                <label class="flex items-center gap-2 w-full cursor-pointer px-2 py-1.5">
                  <Checkbox
                    checked={columnVisibility[col.id] !== false}
                    onchange={() => {
                      columnVisibility = {
                        ...columnVisibility,
                        [col.id]: columnVisibility[col.id] === false
                      };
                    }}
                  />
                  <span class="text-sm">{col.header}</span>
                </label>
              </DropdownItem>
            {/each}
          </Dropdown>
        {/if}

        {#if exportable}
          <Button
            color="alternative"
            size="sm"
            class="gap-1.5"
            onclick={exportCsv}
            disabled={totalFiltered === 0}
          >
            <FileExportOutline class="h-4 w-4" />
            CSV
          </Button>
        {/if}
      </div>
    </div>
  {/if}

  <!-- ═══ Table ═══════════════════════════════════════════════════════════ -->
  <div
    class="overflow-x-auto rounded-xl border border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800 {shadow
      ? 'shadow-sm'
      : ''}"
  >
    <Table {striped} {hoverable}>
      <TableHead>
        {#each visibleColumns as col (col.id)}
          {@const icon = sortIcon(col.id)}
          <TableHeadCell
            class="select-none {col.align === 'right'
              ? 'text-right'
              : col.align === 'center'
              ? 'text-center'
              : ''} {col.sortable !== false ? 'cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700' : ''}"
            onclick={() => toggleSort(col.id)}
          >
            <span class="inline-flex items-center gap-1">
              {col.header}
              {#if col.sortable !== false}
                {#if icon === 'asc'}
                  <ChevronUpOutline class="h-3 w-3" />
                {:else if icon === 'desc'}
                  <ChevronDownOutline class="h-3 w-3" />
                {:else}
                  <ChevronSortOutline class="h-3 w-3 opacity-40" />
                {/if}
              {/if}
            </span>
          </TableHeadCell>
        {/each}
      </TableHead>
      <TableBody>
        {#if displayRows.length === 0}
          <tr>
            <td colspan={visibleColumns.length} class="px-6 py-10 text-center">
              {#if empty}
                {@render empty()}
              {:else}
                <p class="text-sm text-gray-500 dark:text-gray-400">{emptyMessage}</p>
              {/if}
            </td>
          </tr>
        {:else}
          {#each displayRows as item, i (i)}
            {@render row(item, i + pageIndex * pagination.pageSize, visibleColIds)}
          {/each}
        {/if}
      </TableBody>
    </Table>
  </div>

  <!-- ═══ Pagination ══════════════════════════════════════════════════════ -->
  {#if paginated && totalFiltered > 0}
    <div class="flex flex-wrap items-center justify-between gap-3 px-1">
      <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
        <span>Sayfa boyutu:</span>
        <Select
          size="sm"
          value={pagination.pageSize}
          onchange={(e) => setPageSize(Number((e.target as HTMLSelectElement).value))}
          items={pageSizes.map((s) => ({ value: s, name: String(s) }))}
          class="w-20"
        />
        <span class="ml-2">
          {pageIndex * pagination.pageSize + 1}–{Math.min(
            (pageIndex + 1) * pagination.pageSize,
            totalFiltered
          )}
          / {totalFiltered}
        </span>
      </div>

      <div class="flex items-center gap-1">
        <Button
          color="alternative"
          size="xs"
          onclick={goFirst}
          disabled={pageIndex === 0}
          class="px-2"
        >
          «
        </Button>
        <Button
          color="alternative"
          size="xs"
          onclick={goPrev}
          disabled={pageIndex === 0}
          class="px-2"
        >
          <ChevronLeftOutline class="h-3.5 w-3.5" />
        </Button>
        <span class="px-3 text-sm text-gray-600 dark:text-gray-400">
          {pageIndex + 1} / {Math.max(1, pageCount)}
        </span>
        <Button
          color="alternative"
          size="xs"
          onclick={goNext}
          disabled={pageIndex >= pageCount - 1}
          class="px-2"
        >
          <ChevronRightOutline class="h-3.5 w-3.5" />
        </Button>
        <Button
          color="alternative"
          size="xs"
          onclick={goLast}
          disabled={pageIndex >= pageCount - 1}
          class="px-2"
        >
          »
        </Button>
      </div>
    </div>
  {/if}
</div>
