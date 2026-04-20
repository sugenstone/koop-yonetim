import type { ColumnDef as TSColumnDef } from '@tanstack/table-core';

/**
 * DataTable kolon tanımı.
 * - `id`: unique column ID (aynı zamanda accessorKey olarak kullanılır)
 * - `header`: tablo başlığı
 * - `accessor`: değer çıkarıcı (string key veya fn)
 * - `sortable`: sıralama aktif mi (default: true)
 * - `searchable`: global arama'ya dahil mi (default: true)
 * - `width`: min genişlik (opsiyonel, class olarak verilir)
 * - `align`: hücre hizalaması
 */
export interface DataTableColumn<T> {
  id: string;
  header: string;
  accessor: keyof T | ((row: T) => unknown);
  sortable?: boolean;
  searchable?: boolean;
  align?: 'left' | 'center' | 'right';
  hiddenByDefault?: boolean;
}

/**
 * DataTableColumn'u tanstack ColumnDef'e çevirir.
 */
export function toTanstackColumns<T>(cols: DataTableColumn<T>[]): TSColumnDef<T, unknown>[] {
  return cols.map((c) => {
    const def: TSColumnDef<T, unknown> = {
      id: c.id,
      header: c.header,
      enableSorting: c.sortable !== false,
      enableGlobalFilter: c.searchable !== false,
      accessorFn:
        typeof c.accessor === 'function'
          ? (row: T) => (c.accessor as (r: T) => unknown)(row)
          : (row: T) => (row as Record<string, unknown>)[c.accessor as string]
    };
    return def;
  });
}

/**
 * Rows listesini CSV string'ine çevirir (Excel uyumlu UTF-8 BOM ile).
 */
export function rowsToCsv<T>(
  rows: T[],
  columns: DataTableColumn<T>[],
  visibleIds?: Set<string>
): string {
  const cols = visibleIds ? columns.filter((c) => visibleIds.has(c.id)) : columns;
  const escape = (v: unknown): string => {
    if (v == null) return '';
    const s = String(v);
    if (s.includes('"') || s.includes(',') || s.includes('\n') || s.includes(';')) {
      return `"${s.replace(/"/g, '""')}"`;
    }
    return s;
  };
  const header = cols.map((c) => escape(c.header)).join(',');
  const body = rows
    .map((row) =>
      cols
        .map((c) => {
          const val =
            typeof c.accessor === 'function'
              ? (c.accessor as (r: T) => unknown)(row)
              : (row as Record<string, unknown>)[c.accessor as string];
          return escape(val);
        })
        .join(',')
    )
    .join('\n');
  return '\uFEFF' + header + '\n' + body;
}

/**
 * CSV string'ini tarayıcıda dosya olarak indirir.
 */
export function downloadCsv(csv: string, fileName: string): void {
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName.endsWith('.csv') ? fileName : `${fileName}.csv`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
