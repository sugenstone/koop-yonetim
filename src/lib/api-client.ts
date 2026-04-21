/**
 * REST API client. Tauri `invoke()` fonksiyonunun drop-in yerine gecer.
 * Tauri komut adlarini REST endpoint'lerine haritalar.
 */

const API_BASE_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';
const TOKEN_KEY = 'koop_token';

export function getToken(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
	localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
	localStorage.removeItem(TOKEN_KEY);
}

/**
 * JWT payload'unu decode eder (dogrulama yapmaz - sadece okuma).
 * Rol bazli UI gostermek icin kullanilir (gercek yetki backend'de kontrol edilir).
 */
export interface JwtPayload {
	sub: number;
	email: string;
	rol: string;
	exp: number;
}

export function getCurrentUser(): JwtPayload | null {
	const token = getToken();
	if (!token) return null;
	try {
		const parts = token.split('.');
		if (parts.length !== 3) return null;
		const payload = JSON.parse(atob(parts[1].replace(/-/g, '+').replace(/_/g, '/')));
		// Süre kontrolü
		if (payload.exp && payload.exp * 1000 < Date.now()) {
			clearToken();
			return null;
		}
		return payload as JwtPayload;
	} catch {
		return null;
	}
}

export function isAdmin(): boolean {
	return getCurrentUser()?.rol === 'admin';
}

/**
 * Oturumu kapat: token sil + login sayfasına yönlendir.
 */
export function logout(): void {
	clearToken();
	if (typeof localStorage !== 'undefined') {
		localStorage.removeItem('koop_kullanici');
		localStorage.removeItem('koop_izinler');
	}
	if (typeof window !== 'undefined') {
		window.location.href = '/authentication/sign-in';
	}
}

export class ApiError extends Error {
	status: number;
	constructor(message: string, status: number) {
		super(message);
		this.status = status;
	}
}

interface EndpointMapping {
	method: 'GET' | 'POST' | 'PUT' | 'DELETE';
	path: (args: any) => string;
	body?: (args: any) => any;
	query?: (args: any) => Record<string, string | number | null | undefined> | undefined;
}

/**
 * Tauri komut adi -> REST endpoint eslemesi.
 * Not: Bazi komutlar henuz API'de yok (product, user Tauri-special).
 * Onlar cagirildiginda 501 Not Implemented dondurulecek.
 */
const MAPPING: Record<string, EndpointMapping> = {
	// ─── Auth ─────────────────────────────────────────────────────────────
	giris: {
		method: 'POST',
		path: () => '/api/auth/giris',
		body: (a) => ({ email: a.email, sifre: a.sifre })
	},
	kayit: {
		method: 'POST',
		path: () => '/api/auth/kayit',
		body: (a) => ({ ad: a.ad, email: a.email, sifre: a.sifre })
	},

	// ─── Kullanıcı Yönetimi (admin) ───────────────────────────────────────
	get_kullanicilar: { method: 'GET', path: () => '/api/kullanicilar' },
	get_bekleyenler: { method: 'GET', path: () => '/api/kullanicilar/bekleyenler' },
	onayla_kullanici: {
		method: 'POST',
		path: (a) => `/api/kullanicilar/${a.id}/onayla`,
		body: (a) => ({ rol: a.rol })
	},
	reddet_kullanici: {
		method: 'POST',
		path: (a) => `/api/kullanicilar/${a.id}/reddet`
	},
	create_kullanici: { method: 'POST', path: () => '/api/kullanicilar', body: (a) => a.input },
	update_kullanici: {
		method: 'PUT',
		path: (a) => `/api/kullanicilar/${a.input.id}`,
		body: (a) => a.input
	},
	delete_kullanici: { method: 'DELETE', path: (a) => `/api/kullanicilar/${a.id}` },
	change_kullanici_sifre: {
		method: 'PUT',
		path: (a) => `/api/kullanicilar/${a.id}/sifre`,
		body: (a) => a.input
	},

	// ─── İzin/Rol Yönetimi ────────────────────────────────────────────────
	get_izinler: { method: 'GET', path: () => '/api/izinler' },
	get_rol_izinleri: { method: 'GET', path: (a) => `/api/izinler/roller/${a.rol}` },
	set_rol_izinleri: {
		method: 'PUT',
		path: (a) => `/api/izinler/roller/${a.rol}`,
		body: (a) => ({ izin_ids: a.izin_ids })
	},
	get_benim_izinlerim: { method: 'GET', path: () => '/api/izinler/benim' },

	// ─── Admin ────────────────────────────────────────────────────────────
	admin_sifirla: {
		method: 'POST',
		path: () => '/api/admin/sifirla',
		body: (a) => ({ sifre: a.sifre })
	},

	// ─── İşlem Logları (Audit) ────────────────────────────────────────────
	get_loglar: {
		method: 'GET',
		path: (a) => {
			const qs = new URLSearchParams();
			if (a?.limit != null) qs.set('limit', String(a.limit));
			if (a?.offset != null) qs.set('offset', String(a.offset));
			if (a?.kullanici_id != null) qs.set('kullanici_id', String(a.kullanici_id));
			if (a?.yontem) qs.set('yontem', a.yontem);
			if (a?.min_durum != null) qs.set('min_durum', String(a.min_durum));
			if (a?.q) qs.set('q', a.q);
			const s = qs.toString();
			return s ? `/api/loglar?${s}` : '/api/loglar';
		}
	},
	get_log_ozet: { method: 'GET', path: () => '/api/loglar/ozet' },

	// ─── Kasa ─────────────────────────────────────────────────────────────
	get_kasalar: { method: 'GET', path: () => '/api/kasalar' },
	get_kasa: { method: 'GET', path: (a) => `/api/kasalar/${a.id}` },
	create_kasa: { method: 'POST', path: () => '/api/kasalar', body: (a) => a.input },
	update_kasa: {
		method: 'PUT',
		path: (a) => `/api/kasalar/${a.input.id}`,
		body: (a) => a.input
	},
	delete_kasa: { method: 'DELETE', path: (a) => `/api/kasalar/${a.id}` },
	get_kasa_hareketleri: {
		method: 'GET',
		path: (a) => `/api/kasalar/${a.kasaId}/hareketler`
	},
	create_kasa_hareketi: {
		method: 'POST',
		path: (a) => `/api/kasalar/${a.input.kasa_id}/hareketler`,
		body: (a) => a.input
	},
	delete_kasa_hareketi: {
		method: 'DELETE',
		path: (a) => `/api/kasalar/${a.input.kasa_id}/hareketler/${a.input.id}`
	},
	kasa_transfer: {
		method: 'POST',
		path: () => '/api/kasalar/transfer',
		body: (a) => a.input
	},
	get_kasa_transferleri: {
		method: 'GET',
		path: (a) => `/api/kasalar/${a.kasaId}/transferler`
	},

	// ─── Hissedar ─────────────────────────────────────────────────────────
	get_hissedarlar: { method: 'GET', path: () => '/api/hissedarlar' },
	get_hissedar: { method: 'GET', path: (a) => `/api/hissedarlar/${a.id}` },
	create_hissedar: { method: 'POST', path: () => '/api/hissedarlar', body: (a) => a.input },
	update_hissedar: {
		method: 'PUT',
		path: (a) => `/api/hissedarlar/${a.input.id}`,
		body: (a) => a.input
	},
	delete_hissedar: { method: 'DELETE', path: (a) => `/api/hissedarlar/${a.id}` },

	// ─── Donem ────────────────────────────────────────────────────────────
	get_donemler: { method: 'GET', path: () => '/api/donemler' },
	get_donem: { method: 'GET', path: (a) => `/api/donemler/${a.id}` },
	create_donem: { method: 'POST', path: () => '/api/donemler', body: (a) => a.input },
	update_donem: {
		method: 'PUT',
		path: (a) => `/api/donemler/${a.input.id}`,
		body: (a) => a.input
	},
	delete_donem: { method: 'DELETE', path: (a) => `/api/donemler/${a.id}` },

	get_toplantilar: {
		method: 'GET',
		path: (a) => `/api/donemler/${a.donemId}/toplantilar`
	},
	create_toplanti: {
		method: 'POST',
		path: (a) => `/api/donemler/${a.input.donem_id}/toplantilar`,
		body: (a) => a.input
	},
	update_toplanti: {
		method: 'PUT',
		path: (a) => `/api/toplantilar/${a.input.id}`,
		body: (a) => a.input
	},
	delete_toplanti: { method: 'DELETE', path: (a) => `/api/toplantilar/${a.id}` },

	get_kararlar: { method: 'GET', path: (a) => `/api/toplantilar/${a.toplantiId}/kararlar` },
	create_karar: {
		method: 'POST',
		path: (a) => `/api/toplantilar/${a.input.toplanti_id}/kararlar`,
		body: (a) => a.input
	},
	update_karar: {
		method: 'PUT',
		path: (a) => `/api/kararlar/${a.input.id}`,
		body: (a) => a.input
	},
	delete_karar: { method: 'DELETE', path: (a) => `/api/kararlar/${a.id}` },

	// ─── Hisse ────────────────────────────────────────────────────────────
	get_hisseler: { method: 'GET', path: () => '/api/hisseler' },
	get_hisse: { method: 'GET', path: (a) => `/api/hisseler/${a.id}` },
	create_hisse: { method: 'POST', path: () => '/api/hisseler', body: (a) => a.input },
	create_hisse_toplu: {
		method: 'POST',
		path: () => '/api/hisseler/toplu',
		body: (a) => a.input
	},
	delete_hisse: { method: 'DELETE', path: (a) => `/api/hisseler/${a.id}` },

	get_hisse_atamalari: {
		method: 'GET',
		path: (a) => `/api/hisseler/${a.hisseId}/atamalar`
	},
	get_hissedar_atamalari: {
		method: 'GET',
		path: (a) => `/api/hissedarlar/${a.hissedarId}/atamalar`
	},
	hisse_ata: { method: 'POST', path: () => '/api/hisseler/ata', body: (a) => a.input },
	hisse_atama_sil: { method: 'DELETE', path: (a) => `/api/hisseler/atama/${a.id}` },
	hisse_transfer: {
		method: 'POST',
		path: () => '/api/hisseler/transfer',
		body: (a) => a.input
	},

	hisse_satis_baslat: {
		method: 'POST',
		path: () => '/api/hisseler/satis',
		body: (a) => a.input
	},
	hisse_satis_odeme_ekle: {
		method: 'POST',
		path: (a) => `/api/hisseler/satis/${a.input.satis_id}/odeme`,
		body: (a) => a.input
	},
	get_hisse_satis_aktif: {
		method: 'GET',
		path: (a) => `/api/hisseler/${a.hisseId}/satis`
	},
	get_hisse_satis_odemeleri: {
		method: 'GET',
		path: (a) => `/api/hisseler/satis/${a.satisId}/odemeler`
	},
	hisse_satis_iptal: {
		method: 'DELETE',
		path: (a) => `/api/hisseler/satis/${a.satisId}`
	},

	// ─── Aidat ────────────────────────────────────────────────────────────
	donem_borc_olustur: {
		method: 'POST',
		path: (a) => `/api/donemler/${a.donemId}/borclar`
	},
	get_donem_borclari: {
		method: 'GET',
		path: (a) => `/api/donemler/${a.donemId}/borclar`
	},
	get_hisse_borclari: {
		method: 'GET',
		path: (a) => `/api/hisseler/${a.hisseId}/borclar`
	},
	get_hissedar_borclari: {
		method: 'GET',
		path: (a) => `/api/hissedarlar/${a.hissedarId}/borclar`
	},

	// ─── Cuzdan ───────────────────────────────────────────────────────────
	get_hissedar_cuzdani: {
		method: 'GET',
		path: (a) => `/api/hissedarlar/${a.hissedarId}/cuzdan`
	},
	hissedar_cuzdan_para_ekle: {
		method: 'POST',
		path: (a) => `/api/hissedarlar/${a.input.hissedar_id}/cuzdan/para`,
		body: (a) => a.input
	},

	// ─── Gelir/Gider ──────────────────────────────────────────────────────
	get_gelir_gider_kategorileri: {
		method: 'GET',
		path: () => '/api/gelir-gider/kategoriler'
	},
	create_gelir_gider_kategori: {
		method: 'POST',
		path: () => '/api/gelir-gider/kategoriler',
		body: (a) => a.input
	},
	update_gelir_gider_kategori: {
		method: 'PUT',
		path: (a) => `/api/gelir-gider/kategoriler/${a.input.id}`,
		body: (a) => a.input
	},
	delete_gelir_gider_kategori: {
		method: 'DELETE',
		path: (a) => `/api/gelir-gider/kategoriler/${a.id}`
	},
	get_gelir_gider_kayitlari: {
		method: 'GET',
		path: () => '/api/gelir-gider/kayitlar',
		query: (a) => ({ kasa_id: a.kasaId, kategori_id: a.kategoriId })
	},
	create_gelir_gider_kaydi: {
		method: 'POST',
		path: () => '/api/gelir-gider/kayitlar',
		body: (a) => a.input
	},
	delete_gelir_gider_kaydi: {
		method: 'DELETE',
		path: (a) => `/api/gelir-gider/kayitlar/${a.id}`
	}
};

function buildUrl(base: string, path: string, query?: Record<string, any>): string {
	let url = `${base}${path}`;
	if (query) {
		const params = new URLSearchParams();
		for (const [k, v] of Object.entries(query)) {
			if (v !== null && v !== undefined) params.set(k, String(v));
		}
		const qs = params.toString();
		if (qs) url += `?${qs}`;
	}
	return url;
}

/**
 * Tauri `invoke()` icin drop-in yerine.
 * Kullanim: invokeApi('get_kasalar') veya invokeApi('create_kasa', { input: {...} })
 */
export async function invokeApi<T = any>(cmd: string, args: any = {}): Promise<T> {
	const mapping = MAPPING[cmd];
	if (!mapping) {
		throw new ApiError(`Komut haritalanmamis: ${cmd}`, 501);
	}

	const path = mapping.path(args);
	const query = mapping.query ? mapping.query(args) : undefined;
	const url = buildUrl(API_BASE_URL, path, query);

	const headers: Record<string, string> = { 'Content-Type': 'application/json' };
	const token = getToken();
	if (token) headers['Authorization'] = `Bearer ${token}`;

	const init: RequestInit = { method: mapping.method, headers };
	if (mapping.body) {
		init.body = JSON.stringify(mapping.body(args));
	}

	const res = await fetch(url, init);

	if (res.status === 401) {
		clearToken();
		if (typeof window !== 'undefined' && !window.location.pathname.includes('sign-in')) {
			window.location.href = '/authentication/sign-in';
		}
		throw new ApiError('Oturum suresi doldu', 401);
	}

	const text = await res.text();
	const data = text ? JSON.parse(text) : null;

	if (!res.ok) {
		const msg = (data && (data.hata || data.error || data.message)) || `HTTP ${res.status}`;
		const err = new ApiError(msg, res.status);
		// 403 izin hatalarini otomatik toast yap (SSR guard)
		if (res.status === 403 && typeof window !== 'undefined') {
			import('svelte-sonner').then(({ toast }) => {
				toast.warning(msg, { description: 'Bu islem icin yetkiniz yok.' });
			}).catch(() => {});
		}
		throw err;
	}

	return data as T;
}
