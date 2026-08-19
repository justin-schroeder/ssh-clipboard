// Cloudflare Worker: serves the static site and a lightly-authed star count.
// GITHUB_TOKEN is an optional Workers secret (fine-grained PAT, public read);
// with it, GitHub allows 5000 req/hr — and the edge cache means we use ~6.
const CACHE_SECONDS = 600

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url)
    if (url.pathname === '/api/stars') {
      return stars(request, env, ctx)
    }
    if (url.pathname === '/api/early-access' && request.method === 'POST') {
      return earlyAccess(request, env)
    }
    return env.ASSETS.fetch(request)
  },
}

// Forwards signups to the shared Standard Agents waitlist (same backend dmux.ai uses).
async function earlyAccess(request, env) {
  const body = await request.json().catch(() => null)
  const name = body?.name?.trim()
  const email = body?.email?.trim()
  if (!name || !email) {
    return Response.json({ error: 'Name and email are required.' }, { status: 400 })
  }
  try {
    const headers = { 'Content-Type': 'application/json' }
    if (env.WAITLIST_API_TOKEN) {
      headers.Authorization = `Bearer ${env.WAITLIST_API_TOKEN}`
    }
    const upstream = await fetch('https://dmux.ai/api/early-access', {
      method: 'POST',
      headers,
      body: JSON.stringify({ name, email }),
    })
    if (!upstream.ok) {
      return Response.json({ error: 'Signup failed. Please try again.' }, { status: 502 })
    }
    return Response.json({ ok: true })
  } catch {
    return Response.json({ error: 'Signup failed. Please try again.' }, { status: 502 })
  }
}

async function stars(request, env, ctx) {
  const cache = caches.default
  const cacheKey = new Request(new URL('/api/stars', request.url), { method: 'GET' })
  const hit = await cache.match(cacheKey)
  if (hit) return hit

  const headers = {
    'User-Agent': 'ssh-clipboard-site',
    Accept: 'application/vnd.github+json',
  }
  if (env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${env.GITHUB_TOKEN}`
  }

  let count = null
  try {
    const res = await fetch('https://api.github.com/repos/standardagents/ssh-clipboard', { headers })
    if (res.ok) {
      const data = await res.json()
      if (typeof data.stargazers_count === 'number') count = data.stargazers_count
    }
  } catch {
    /* count stays null */
  }

  const response = Response.json(
    { stars: count },
    {
      headers: {
        'cache-control': `public, max-age=300, s-maxage=${CACHE_SECONDS}`,
        'access-control-allow-origin': '*',
      },
    }
  )
  if (count !== null) {
    ctx.waitUntil(cache.put(cacheKey, response.clone()))
  }
  return response
}
