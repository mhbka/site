import type { AstroCookies } from 'astro';

import { blogApi } from './api.ts';
import { createSupabaseServerClient } from './auth/supabase.ts';

interface Options {
	includeAuthor?: boolean;
	includePix?: boolean;
}

export async function getPageAuth(request: Request, cookies: AstroCookies, { includeAuthor = false, includePix = false }: Options = {}) {
	const supabaseClient = createSupabaseServerClient(request, cookies);
	const { data: { session } } = await supabaseClient.auth.getSession();

	if (!session) return { isLoggedIn: false, isAuthor: false, isPix: false, accessToken: undefined };
	if (!includeAuthor && !includePix) return { isLoggedIn: true, isAuthor: false, isPix: false, accessToken: session.access_token };

	try {
		return {
			isLoggedIn: true,
			isAuthor: includeAuthor ? (await blogApi.getAuthorStatus(session.access_token)).isAuthor : false,
			isPix: includePix ? (await blogApi.getPixStatus(session.access_token)).isPix : false,
			accessToken: session.access_token,
		};
	} catch (error) {
		console.warn(`Unable to verify user access: ${error}`);
		return { isLoggedIn: true, isAuthor: false, isPix: false, accessToken: session.access_token };
	}
}
