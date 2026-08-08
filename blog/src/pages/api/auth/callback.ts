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
    else console.log(`Error authing: ${error}`)
  }
  else console.log('No code obtained during auth')

  return redirect('/auth/auth-code-error')
}