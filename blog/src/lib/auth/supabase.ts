import { createBrowserClient, createServerClient, parseCookieHeader } from "@supabase/ssr"
import type { AstroCookies } from "astro"

/** Creates the server-side Supabase client with Astro cookie support. */
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

/** Creates the browser-side Supabase client for interactive components. */
export const createSupabaseBrowserClient = () => {
  return createBrowserClient(
    import.meta.env.SUPABASE_URL,
    import.meta.env.SUPABASE_PUBLISHABLE_KEY,
    {
      auth: {
        flowType: 'pkce',
      },
    },
  )
}
