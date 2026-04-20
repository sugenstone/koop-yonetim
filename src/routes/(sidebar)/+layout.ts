import { redirect } from '@sveltejs/kit';
import { getToken } from '$lib/api-client';

export const prerender = false;
export const ssr = false;

export const load = async ({ url }) => {
	// Tarayici tarafi - token kontrolu
	if (typeof localStorage !== 'undefined') {
		const token = getToken();
		if (!token) {
			throw redirect(302, `/authentication/sign-in?next=${encodeURIComponent(url.pathname)}`);
		}
	}
	return {};
};
