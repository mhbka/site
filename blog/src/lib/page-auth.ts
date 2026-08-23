import type { AstroCookies } from 'astro';

import { blogApi } from './api.ts';
import { createSupabaseServerClient } from './auth/supabase.ts';

interface Options {
	includeAuthor?: boolean;
}

export async function getPageAuth(request: Request, cookies: AstroCookies, { includeAuthor = false }: Options = {}) {
	const supabaseClient = createSupabaseServerClient(request, cookies);
	const { data: { session } } = await supabaseClient.auth.getSession();

	if (!session) return { isLoggedIn: false, isAuthor: false, accessToken: undefined };
	if (!includeAuthor) return { isLoggedIn: true, isAuthor: false, accessToken: session.access_token };

	try {
		return {
			isLoggedIn: true,
			isAuthor: (await blogApi.getAuthorStatus(session.access_token)).isAuthor,
			accessToken: session.access_token,
		};
	} catch (error) {
		console.warn(`Unable to verify author access: ${error}`);
		return { isLoggedIn: true, isAuthor: false, accessToken: session.access_token };
	}
}
