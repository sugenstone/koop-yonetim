/**
 * Merkezi toast bildirim yardimcisi.
 * svelte-sonner uzerine ince bir katman.
 *
 * Kullanim:
 *   import { notify } from '$lib/toast';
 *   notify.success('Kaydedildi');
 *   notify.error('Hata olustu');
 *   notify.forbidden('kasa.olustur');  // Izin hatasi
 */

import { toast } from 'svelte-sonner';

export const notify = {
	success(msg: string) {
		toast.success(msg);
	},
	error(msg: string) {
		toast.error(msg);
	},
	info(msg: string) {
		toast.info(msg);
	},
	warning(msg: string) {
		toast.warning(msg);
	},

	/**
	 * Backend'den gelen ApiError veya genel hata icin otomatik secim.
	 * 403 -> uyari, digerleri -> hata.
	 */
	apiError(e: unknown, fallback = 'Islem basarisiz') {
		const err = e as { status?: number; message?: string };
		const msg = err?.message || String(e) || fallback;
		if (err?.status === 403) {
			toast.warning(msg, { description: 'Bu islem icin yetkiniz yok.' });
		} else if (err?.status === 401) {
			toast.error('Oturum suresi doldu, tekrar giris yapin.');
		} else {
			toast.error(msg);
		}
	},

	/**
	 * UI tarafinda izin yoksa tetiklenecek mesaj.
	 */
	forbidden(anahtar?: string) {
		toast.warning(
			anahtar ? `'${anahtar}' izniniz yok` : 'Bu islem icin yetkiniz yok',
			{ description: 'Yoneticinizle iletisime gecin.' }
		);
	}
};
