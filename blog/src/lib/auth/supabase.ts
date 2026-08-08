import { createBrowserClient, createServerClient, parseCookieHeader } from "@supabase/ssr"
import type { AstroCookies } from "astro"

// Server-side client, for APIs etc to use
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

// Browser-side client
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
