import type { Hissedar } from './tauri-api';

/**
 * Hissedar bilgisini tek satırda "Ad Soyad (Yakınlık: Yakın Adı)" formatında döndürür.
 * Yakın bilgisi yoksa sadece "Ad Soyad" döner.
 */
export function hissedarLabel(h: {
  ad: string;
  soyad: string;
  yakin_adi?: string | null;
  yakinlik_derecesi?: string | null;
}): string {
  const base = `${h.ad} ${h.soyad}`;
  if (h.yakin_adi && h.yakinlik_derecesi) {
    return `${base} (${h.yakinlik_derecesi}: ${h.yakin_adi})`;
  }
  if (h.yakin_adi) {
    return `${base} (${h.yakin_adi})`;
  }
  return base;
}

/**
 * hissedar_id ile bir listeden hissedarı bulup formatlı label döndürür.
 * Bulunamazsa fallback parametresi döner.
 */
export function hissedarLabelFromId(
  id: number | undefined | null,
  hissedarlar: Hissedar[],
  fallback = '—'
): string {
  if (id == null) return fallback;
  const h = hissedarlar.find((x) => x.id === id);
  return h ? hissedarLabel(h) : fallback;
}

/**
 * Ham alanlardan (API response'larındaki hissedar_ad/hissedar_soyad) label üretir.
 * Lookup listesi varsa yakın bilgisi de eklenir.
 */
export function hissedarLabelFromFields(
  hissedar_id: number | undefined | null,
  hissedar_ad: string | undefined | null,
  hissedar_soyad: string | undefined | null,
  hissedarlar?: Hissedar[]
): string {
  if (hissedar_id != null && hissedarlar) {
    const h = hissedarlar.find((x) => x.id === hissedar_id);
    if (h) return hissedarLabel(h);
  }
  if (hissedar_ad && hissedar_soyad) {
    return `${hissedar_ad} ${hissedar_soyad}`;
  }
  return '—';
}
