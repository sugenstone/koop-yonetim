/**
 * Izin bazli UI kontrolu.
 * Login sonrasi /api/izinler/benim endpoint'inden kullanicinin izin anahtarlari cekilir.
 * Admin rolu her zaman "bypass" olarak kabul edilir (backend ile ayni davranis).
 *
 * Kullanim:
 *   import { loadMyPermissions, hasPermission } from '$lib/permissions';
 *   await loadMyPermissions();          // giris sonrasi veya app mount'ta
 *   if (hasPermission('kasa.olustur')) { ... }
 *
 * Svelte componentlerinde reaktif kullanim:
 *   import { myPermissions } from '$lib/permissions';
 *   $myPermissions.has('kasa.olustur')
 */

import { writable, derived, get } from 'svelte/store';
import { invokeApi, getCurrentUser } from './api-client';

export type PermissionSet = Set<string>;

const STORAGE_KEY = 'koop_izinler';

function readCache(): PermissionSet {
	if (typeof localStorage === 'undefined') return new Set();
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return new Set();
		const arr = JSON.parse(raw);
		return new Set(Array.isArray(arr) ? arr : []);
	} catch {
		return new Set();
	}
}

function writeCache(set: PermissionSet) {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(Array.from(set)));
}

/** Kullanicinin izinleri (anahtar seti). */
export const myPermissions = writable<PermissionSet>(readCache());

/** Turevsel: admin mi? (rol token'dan okunur) */
export const isAdminStore = derived(myPermissions, () => getCurrentUser()?.rol === 'admin');

/**
 * API'den izinleri cek ve store'u guncelle.
 * Hata olursa cache korunur.
 */
export async function loadMyPermissions(): Promise<void> {
	const u = getCurrentUser();
	if (!u) {
		myPermissions.set(new Set());
		writeCache(new Set());
		return;
	}
	// Admin icin bos set tutabiliriz, hasPermission zaten bypass eder.
	try {
		const list = await invokeApi<string[]>('get_benim_izinlerim');
		const set = new Set(Array.isArray(list) ? list : []);
		myPermissions.set(set);
		writeCache(set);
	} catch (e) {
		console.warn('Izinler yuklenemedi, cache kullaniliyor:', e);
	}
}

export function clearPermissions(): void {
	myPermissions.set(new Set());
	if (typeof localStorage !== 'undefined') {
		localStorage.removeItem(STORAGE_KEY);
	}
}

/**
 * Tek seferlik (reaktif olmayan) kontrol. Admin her zaman true dondurur.
 */
export function hasPermission(anahtar: string): boolean {
	const u = getCurrentUser();
	if (!u) return false;
	if (u.rol === 'admin') return true;
	return get(myPermissions).has(anahtar);
}

/**
 * Birden fazla izinden herhangi birine sahipse true.
 */
export function hasAnyPermission(...anahtarlar: string[]): boolean {
	return anahtarlar.some((a) => hasPermission(a));
}

/**
 * Tum izinlere sahipse true.
 */
export function hasAllPermissions(...anahtarlar: string[]): boolean {
	return anahtarlar.every((a) => hasPermission(a));
}
