/**
 * Tauri backend ile iletişim için API helper'ları.
 * Rust komutlarını (invoke) çağırır ve SQLite verilerini döner.
 */

import { invoke } from '@tauri-apps/api/core';

// ─── Tip Tanımları ────────────────────────────────────────────────────────────

export interface User {
	id: number;
	name: string;
	email: string;
	role: string;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface CreateUserInput {
	name: string;
	email: string;
	role?: string;
}

export interface UpdateUserInput {
	id: number;
	name?: string;
	email?: string;
	role?: string;
	status?: string;
}

export interface Product {
	id: number;
	name: string;
	category: string;
	price: number;
	stock: number;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface CreateProductInput {
	name: string;
	category: string;
	price: number;
	stock?: number;
}

export interface UpdateProductInput {
	id: number;
	name?: string;
	category?: string;
	price?: number;
	stock?: number;
	status?: string;
}

export interface DashboardStats {
	total_users: number;
	total_products: number;
	active_users: number;
	active_products: number;
}

// ─── Kullanıcı API'leri ───────────────────────────────────────────────────────

export const userApi = {
	getAll: (): Promise<User[]> => invoke('get_users'),

	create: (input: CreateUserInput): Promise<User> => invoke('create_user', { input }),

	update: (input: UpdateUserInput): Promise<User> => invoke('update_user', { input }),

	delete: (id: number): Promise<void> => invoke('delete_user', { id })
};

// ─── Ürün API'leri ────────────────────────────────────────────────────────────

export const productApi = {
	getAll: (): Promise<Product[]> => invoke('get_products'),

	create: (input: CreateProductInput): Promise<Product> => invoke('create_product', { input }),

	update: (input: UpdateProductInput): Promise<Product> => invoke('update_product', { input }),

	delete: (id: number): Promise<void> => invoke('delete_product', { id })
};

// ─── Dashboard API'leri ───────────────────────────────────────────────────────

export const dashboardApi = {
	getStats: (): Promise<DashboardStats> => invoke('get_dashboard_stats')
};

// ─── Tauri ortamında mı çalışıyor? ───────────────────────────────────────────

export const isTauri = (): boolean => {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// ─── Kasa Tipleri ─────────────────────────────────────────────────────────────

export type ParaBirimi = 'TL' | 'USD' | 'EUR' | 'ALTIN';

export interface Kasa {
	id: number;
	ad: string;
	para_birimi: ParaBirimi;
	bakiye: number;
	aciklama?: string;
	aktif: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateKasaInput {
	ad: string;
	para_birimi: ParaBirimi;
	aciklama?: string;
}

export interface UpdateKasaInput {
	id: number;
	ad?: string;
	para_birimi?: ParaBirimi;
	aciklama?: string;
	aktif?: boolean;
}

export interface KasaHareketi {
	id: number;
	kasa_id: number;
	tarih: string;
	aciklama: string;
	giren: number;
	cikan: number;
	bakiye: number;
	created_at: string;
}

export interface CreateHareket {
	kasa_id: number;
	tarih: string;
	aciklama: string;
	giren: number;
	cikan: number;
}

export interface DeleteHareket {
	id: number;
	kasa_id: number;
}

// ─── Kasa API'leri ────────────────────────────────────────────────────────────

export const kasaApi = {
	getAll: (): Promise<Kasa[]> => invoke('get_kasalar'),
	get: (id: number): Promise<Kasa> => invoke('get_kasa', { id }),
	create: (input: CreateKasaInput): Promise<Kasa> => invoke('create_kasa', { input }),
	update: (input: UpdateKasaInput): Promise<Kasa> => invoke('update_kasa', { input }),
	delete: (id: number): Promise<void> => invoke('delete_kasa', { id })
};

export const kasaHareketiApi = {
	getAll: (kasaId: number): Promise<KasaHareketi[]> =>
		invoke('get_kasa_hareketleri', { kasaId: kasaId }),
	create: (input: CreateHareket): Promise<KasaHareketi> =>
		invoke('create_kasa_hareketi', { input }),
	delete: (input: DeleteHareket): Promise<void> => invoke('delete_kasa_hareketi', { input })
};

// ─── Kasa Transfer Tipleri ────────────────────────────────────────────────────

export interface KasaTransfer {
	id: number;
	kaynak_kasa_id: number;
	kaynak_kasa_ad: string;
	kaynak_kasa_para_birimi: string;
	hedef_kasa_id: number;
	hedef_kasa_ad: string;
	hedef_kasa_para_birimi: string;
	tarih: string;
	kaynak_miktar: number;
	hedef_miktar: number;
	kur?: number;
	aciklama?: string;
	created_at: string;
}

export interface TransferInput {
	kaynak_kasa_id: number;
	hedef_kasa_id: number;
	tarih: string;
	/** Hedef kasaya eklenecek miktar (aynı para biriminde hem kaynak hem hedef) */
	hedef_miktar: number;
	/** Farklı para birimlerinde: 1 hedef birimi = kur kaynak birimi */
	kur?: number;
	aciklama?: string;
}

export const kasaTransferApi = {
	create: (input: TransferInput): Promise<KasaTransfer> => invoke('kasa_transfer', { input }),
	getAll: (kasaId: number): Promise<KasaTransfer[]> =>
		invoke('get_kasa_transferleri', { kasaId })
};

// ─── Para birimi yardımcıları ─────────────────────────────────────────────────

export const PARA_BIRIMLERI: { value: ParaBirimi; label: string; sembol: string }[] = [
	{ value: 'TL', label: 'Türk Lirası', sembol: '₺' },
	{ value: 'USD', label: 'Amerikan Doları', sembol: '$' },
	{ value: 'EUR', label: 'Euro', sembol: '€' },
	{ value: 'ALTIN', label: 'Altın (gram)', sembol: 'gr' }
];

export function paraSembol(birimi: ParaBirimi): string {
	return PARA_BIRIMLERI.find((p) => p.value === birimi)?.sembol ?? birimi;
}

export function formatBakiye(miktar: number, birimi: ParaBirimi): string {
	if (birimi === 'ALTIN') {
		return `${miktar.toFixed(4)} gr`;
	}
	return new Intl.NumberFormat('tr-TR', {
		style: 'currency',
		currency: birimi === 'USD' ? 'USD' : birimi === 'EUR' ? 'EUR' : 'TRY',
		minimumFractionDigits: 2
	}).format(miktar);
}

// ─── Hissedar Tipleri ─────────────────────────────────────────────────────────

export interface Hissedar {
	id: number;
	ad: string;
	soyad: string;
	kasa_id: number;
	kasa_ad?: string;
	aile_sira_no?: number;
	tcno?: string;
	tel?: string;
	yakin_adi?: string;
	yakinlik_derecesi?: string;
	aktif: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateHissedarInput {
	ad: string;
	soyad: string;
	kasa_id: number;
	aile_sira_no?: number;
	tcno?: string;
	tel?: string;
	yakin_adi?: string;
	yakinlik_derecesi?: string;
}

export interface UpdateHissedarInput {
	id: number;
	ad?: string;
	soyad?: string;
	kasa_id?: number;
	aile_sira_no?: number;
	tcno?: string;
	tel?: string;
	yakin_adi?: string;
	yakinlik_derecesi?: string;
	aktif?: boolean;
}

// ─── Hissedar API ─────────────────────────────────────────────────────────────

export const hissedarApi = {
	getAll: (): Promise<Hissedar[]> => invoke('get_hissedarlar'),
	get: (id: number): Promise<Hissedar> => invoke('get_hissedar', { id }),
	create: (input: CreateHissedarInput): Promise<Hissedar> => invoke('create_hissedar', { input }),
	update: (input: UpdateHissedarInput): Promise<Hissedar> => invoke('update_hissedar', { input }),
	delete: (id: number): Promise<void> => invoke('delete_hissedar', { id })
};

// ─── Dönem Tipleri ────────────────────────────────────────────────────────────

export interface Donem {
	id: number;
	ay: number;
	yil: number;
	hisse_basi_aidat: number;
	aktif: boolean;
	toplanti_sayisi: number;
	created_at: string;
	updated_at: string;
}

export interface Toplanti {
	id: number;
	donem_id: number;
	tarih: string;
	konu: string;
	yer?: string;
	karar_sayisi: number;
	created_at: string;
	updated_at: string;
}

export interface Karar {
	id: number;
	toplanti_id: number;
	karar_no?: number;
	aciklama: string;
	created_at: string;
}

export interface CreateDonemInput {
	ay: number;
	yil: number;
	hisse_basi_aidat: number;
}

export interface UpdateDonemInput {
	id: number;
	ay?: number;
	yil?: number;
	hisse_basi_aidat?: number;
	aktif?: boolean;
}

export interface CreateToplantIInput {
	donem_id: number;
	tarih: string;
	konu: string;
	yer?: string;
}

export interface UpdateToplantIInput {
	id: number;
	tarih?: string;
	konu?: string;
	yer?: string;
}

export interface CreateKararInput {
	toplanti_id: number;
	karar_no?: number;
	aciklama: string;
}

export interface UpdateKararInput {
	id: number;
	karar_no?: number;
	aciklama?: string;
}

// ─── Dönem Yardımcıları ───────────────────────────────────────────────────────

export const AY_ADLARI = [
	'OCAK', 'ŞUBAT', 'MART', 'NİSAN', 'MAYIS', 'HAZİRAN',
	'TEMMUZ', 'AĞUSTOS', 'EYLÜL', 'EKİM', 'KASIM', 'ARALIK'
];

export function donemAdi(ay: number, yil: number): string {
	return `${AY_ADLARI[ay - 1]} ${yil}`;
}

export function donemYillari(): number[] {
	const current = new Date().getFullYear();
	return Array.from({ length: 11 }, (_, i) => current - 5 + i);
}

// ─── Dönem API ────────────────────────────────────────────────────────────────

export const donemApi = {
	getAll: (): Promise<Donem[]> => invoke('get_donemler'),
	get: (id: number): Promise<Donem> => invoke('get_donem', { id }),
	create: (input: CreateDonemInput): Promise<Donem> => invoke('create_donem', { input }),
	update: (input: UpdateDonemInput): Promise<Donem> => invoke('update_donem', { input }),
	delete: (id: number): Promise<void> => invoke('delete_donem', { id })
};

export const toplantIApi = {
	getAll: (donemId: number): Promise<Toplanti[]> => invoke('get_toplantilar', { donemId }),
	create: (input: CreateToplantIInput): Promise<Toplanti> => invoke('create_toplanti', { input }),
	update: (input: UpdateToplantIInput): Promise<Toplanti> => invoke('update_toplanti', { input }),
	delete: (id: number): Promise<void> => invoke('delete_toplanti', { id })
};

export const kararApi = {
	getAll: (toplantiId: number): Promise<Karar[]> => invoke('get_kararlar', { toplantiId }),
	create: (input: CreateKararInput): Promise<Karar> => invoke('create_karar', { input }),
	update: (input: UpdateKararInput): Promise<Karar> => invoke('update_karar', { input }),
	delete: (id: number): Promise<void> => invoke('delete_karar', { id })
};

// ─── Hisse Tipleri ───────────────────────────────────────────────────────────

export type HisseDurum = 'musait' | 'atanmis' | 'satildi';

export interface Hisse {
	id: number;
	kod: string;
	durum: HisseDurum;
	aciklama?: string;
	created_at: string;
	updated_at: string;
	hissedar_id?: number;
	hissedar_ad?: string;
	hissedar_soyad?: string;
}

export interface HisseAtama {
	id: number;
	hisse_id: number;
	hisse_kod: string;
	hissedar_id: number;
	hissedar_ad: string;
	hissedar_soyad: string;
	tarih: string;
	ucret: number;
	aciklama?: string;
	created_at: string;
}

export interface CreateHisseInput {
	aciklama?: string;
	atama_hissedar_id?: number;
	atama_tarih?: string;
	atama_ucret?: number;
	atama_aciklama?: string;
}

export interface CreateHisseTopluInput {
	adet: number;
	aciklama?: string;
	atama_hissedar_id?: number;
	atama_tarih?: string;
	atama_ucret?: number;
	atama_aciklama?: string;
}

export interface AtamaInput {
	hisse_id: number;
	hissedar_id: number;
	tarih: string;
	ucret: number;
	aciklama?: string;
}

export interface HisseTransferInput {
	hisse_id: number;
	yeni_hissedar_id: number;
	tarih: string;
	ucret: number;
	aciklama?: string;
}

// ─── Hisse API'leri ──────────────────────────────────────────────────────────

export const hisseApi = {
	getAll: (): Promise<Hisse[]> => invoke('get_hisseler'),
	get: (id: number): Promise<Hisse> => invoke('get_hisse', { id }),
	create: (input: CreateHisseInput): Promise<Hisse> => invoke('create_hisse', { input }),
	createToplu: (input: CreateHisseTopluInput): Promise<Hisse[]> =>
		invoke('create_hisse_toplu', { input }),
	delete: (id: number): Promise<void> => invoke('delete_hisse', { id })
};

export const hisseAtamaApi = {
	getByHisse: (hisseId: number): Promise<HisseAtama[]> =>
		invoke('get_hisse_atamalari', { hisseId }),
	getByHissedar: (hissedarId: number): Promise<HisseAtama[]> =>
		invoke('get_hissedar_atamalari', { hissedarId }),
	ata: (input: AtamaInput): Promise<HisseAtama> => invoke('hisse_ata', { input }),
	sil: (id: number): Promise<void> => invoke('hisse_atama_sil', { id }),
	transfer: (input: HisseTransferInput): Promise<HisseAtama> =>
		invoke('hisse_transfer', { input })
};

// ─── Hisse Satışı (Hissedar → Sistem) ────────────────────────────────────────

export interface HisseSatis {
	id: number;
	hisse_id: number;
	hisse_kod: string;
	hissedar_id: number;
	hissedar_ad: string;
	hissedar_soyad: string;
	kasa_id: number;
	kasa_ad: string;
	satis_tutari: number;
	odenen_tutar: number;
	kalan_tutar: number;
	tarih: string;
	tamamlandi: boolean;
	tamamlanma_tarihi?: string;
	iptal: boolean;
	aciklama?: string;
	created_at: string;
}

export interface HisseSatisOdeme {
	id: number;
	satis_id: number;
	tutar: number;
	tarih: string;
	aciklama?: string;
	created_at: string;
}

export interface HisseSatisBaslatInput {
	hisse_id: number;
	kasa_id: number;
	satis_tutari: number;
	tarih: string;
	aciklama?: string;
}

export interface HisseSatisOdemeInput {
	satis_id: number;
	tutar: number;
	tarih: string;
	aciklama?: string;
}

export const hisseSatisApi = {
	baslat: (input: HisseSatisBaslatInput): Promise<HisseSatis> =>
		invoke('hisse_satis_baslat', { input }),
	odemeEkle: (input: HisseSatisOdemeInput): Promise<HisseSatis> =>
		invoke('hisse_satis_odeme_ekle', { input }),
	getAktif: (hisseId: number): Promise<HisseSatis | null> =>
		invoke('get_hisse_satis_aktif', { hisseId }),
	getOdemeler: (satisId: number): Promise<HisseSatisOdeme[]> =>
		invoke('get_hisse_satis_odemeleri', { satisId }),
	iptal: (satisId: number): Promise<void> => invoke('hisse_satis_iptal', { satisId })
};

// ─── Aidat Borç ──────────────────────────────────────────────────────────────

export interface AidatBorcu {
	id: number;
	donem_id: number;
	donem_adi: string;
	hissedar_id: number;
	hissedar_ad: string;
	hissedar_soyad: string;
	hisse_sayisi: number;
	tutar: number;
	odendi: boolean;
	odeme_tarihi?: string;
	aciklama?: string;
	created_at: string;
}

export interface BorcOlusturSonuc {
	olusturulan: number;
	otomatik_tahsil: number;
	tahsil_edilemeyen: number;
}

export const aidatApi = {
	borcOlustur: (donemId: number): Promise<BorcOlusturSonuc> =>
		invoke('donem_borc_olustur', { donemId }),
	getByDonem: (donemId: number): Promise<AidatBorcu[]> =>
		invoke('get_donem_borclari', { donemId }),
	getByHisse: (hisseId: number): Promise<AidatBorcu[]> =>
		invoke('get_hisse_borclari', { hisseId }),
	getByHissedar: (hissedarId: number): Promise<AidatBorcu[]> =>
		invoke('get_hissedar_borclari', { hissedarId }),
};

// ─── Cüzdan Tipleri ──────────────────────────────────────────────────────────

export interface CuzdanHareketi {
	id: number;
	hissedar_id: number;
	donem_id?: number;
	donem_adi?: string;
	tarih: string;
	bilgi: string;
	borc: number;
	alacak: number;
	bakiye: number;
	created_at: string;
}

export interface CuzdanParaEkleInput {
	hissedar_id: number;
	tutar: number;
	aciklama?: string;
}

export interface CuzdanParaEkleSonuc {
	yeni_bakiye: number;
	tahsil_edilen_borc_sayisi: number;
	tahsil_edilen_toplam: number;
}

// ─── Cüzdan API ──────────────────────────────────────────────────────────────

export const cuzdanApi = {
	getByHissedar: (hissedarId: number): Promise<CuzdanHareketi[]> =>
		invoke('get_hissedar_cuzdani', { hissedarId }),
	paraEkle: (input: CuzdanParaEkleInput): Promise<CuzdanParaEkleSonuc> =>
		invoke('hissedar_cuzdan_para_ekle', { input }),
};

// ─── Gelir/Gider Tipleri ─────────────────────────────────────────────────────

export type GelirGiderTip = 'gelir' | 'gider';

export interface GelirGiderKategori {
	id: number;
	ad: string;
	tip: GelirGiderTip;
	aciklama?: string;
	aktif: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateKategoriInput {
	ad: string;
	tip: GelirGiderTip;
	aciklama?: string;
}

export interface UpdateKategoriInput {
	id: number;
	ad?: string;
	aciklama?: string;
	aktif?: boolean;
}

export interface GelirGiderKayit {
	id: number;
	kasa_id: number;
	kasa_ad: string;
	kategori_id: number;
	kategori_ad: string;
	kategori_tip: GelirGiderTip;
	tarih: string;
	tutar: number;
	aciklama: string;
	kasa_hareketi_id?: number;
	created_at: string;
}

export interface CreateKayitInput {
	kasa_id: number;
	kategori_id: number;
	tarih: string;
	tutar: number;
	aciklama: string;
}

// ─── Gelir/Gider API'leri ─────────────────────────────────────────────────────

export const gelirGiderKategoriApi = {
	getAll: (): Promise<GelirGiderKategori[]> => invoke('get_gelir_gider_kategorileri'),
	create: (input: CreateKategoriInput): Promise<GelirGiderKategori> =>
		invoke('create_gelir_gider_kategori', { input }),
	update: (input: UpdateKategoriInput): Promise<GelirGiderKategori> =>
		invoke('update_gelir_gider_kategori', { input }),
	delete: (id: number): Promise<void> => invoke('delete_gelir_gider_kategori', { id }),
};

export const gelirGiderApi = {
	getAll: (kasaId?: number, kategoriId?: number): Promise<GelirGiderKayit[]> =>
		invoke('get_gelir_gider_kayitlari', { kasaId: kasaId ?? null, kategoriId: kategoriId ?? null }),
	create: (input: CreateKayitInput): Promise<GelirGiderKayit> =>
		invoke('create_gelir_gider_kaydi', { input }),
	delete: (id: number): Promise<void> => invoke('delete_gelir_gider_kaydi', { id }),
};
