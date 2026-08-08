import { createServerClient, parseCookieHeader } from "@supabase/ssr"
import { createClient } from "@supabase/supabase-js"
import type { AstroCookies } from "astro"

export const createSupabaseServerClient = (request: Request, cookies: AstroCookies) => {
  return createServerClient(
      import.meta.env.SUPABASE_URL,
      import.meta.env.SUPABASE_PUBLISHABLE_KEY,
      {
        cookies: {
          getAll() {
            return parseCookieHeader(request.headers.get('Cookie') ?? '')
          },
          setAll(cookiesToSet, _headers) {
            cookiesToSet.forEach(({ name, value, options }) => cookies.set(name, value, options))
          },
        },
      }
    )
}

export const supabaseClient = createClient(
  import.meta.env.SUPABASE_URL,
  import.meta.env.SUPABASE_PUBLISHABLE_KEY,
);
