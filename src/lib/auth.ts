/**
 * JWT token ve kullanıcı oturumu yönetimi.
 * localStorage tabanlı, hem Tauri hem de browser ortamında çalışır.
 */

const TOKEN_KEY = 'koop_token';
const USER_KEY = 'koop_user';

export interface AuthUser {
	id: number;
	ad: string;
	email: string;
	rol: string;
	aktif: boolean;
	created_at: string;
}

export function getToken(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
	localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
	localStorage.removeItem(TOKEN_KEY);
	localStorage.removeItem(USER_KEY);
}

export function getUser(): AuthUser | null {
	if (typeof localStorage === 'undefined') return null;
	const u = localStorage.getItem(USER_KEY);
	if (!u) return null;
	try {
		return JSON.parse(u);
	} catch {
		return null;
	}
}

export function setUser(user: AuthUser): void {
	localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export function isLoggedIn(): boolean {
	return !!getToken();
}

export function logout(): void {
	clearToken();
	window.location.href = '/login';
}
