import { type APIRoute } from 'astro'
import { createSupabaseServerClient } from '../../../lib/auth/supabase'
export const GET: APIRoute = async ({ request, cookies, redirect }) => {
  const requestUrl = new URL(request.url)
  const code = requestUrl.searchParams.get('code')
  const next = requestUrl.searchParams.get('next') || '/'
  if (code) {
    const supabaseServerClient = createSupabaseServerClient(request, cookies)
    const { error } = await supabaseServerClient.auth.exchangeCodeForSession(code)
    if (!error) {
      return redirect(next)
    }
  }
  // return the user to an error page with instructions
  return redirect('/auth/auth-code-error')
}