# ssh-clipboard website

The marketing/docs site for ssh-clipboard. Vue 3 + Vite 8, statically prerendered
at build time (no client-side data fetching; the JS only hydrates the copy buttons).

## Develop

```sh
cd docs
pnpm install
pnpm run dev
```

## Build

```sh
pnpm run build   # vite build → SSR bundle → prerender → dist/
```

`dist/index.html` contains the fully rendered page.

## Deploy (Cloudflare Workers Builds only)

The site deploys via **Cloudflare Workers Builds** — Cloudflare runs the build
itself on every push to `main`; GitHub Actions is not involved. One-time setup in
the Cloudflare dashboard:

1. Workers & Pages → Create → **Workers** → Import a repository →
   `standardagents/ssh-clipboard`.
2. Set **Root directory** to `docs`.
3. Set **Build command** to `pnpm install --frozen-lockfile && pnpm run build`.
4. Set **Deploy command** to `pnpm exec wrangler deploy` (it picks up `wrangler.jsonc`,
   which serves `dist/` as static assets).

After that, every push to `main` that touches `docs/` triggers a build and deploy.
(In the Worker's Build settings you can add a watch path of `docs/*` to skip
builds for unrelated pushes.)

Production deployments must originate from the connected Git repository through
Cloudflare Workers Builds. Do not run a production `wrangler deploy` locally;
local credentials are for read-only diagnostics and secret administration.

## GitHub star count

The nav's star button reads `/api/stars`, served by `worker/index.js` — the
Worker queries the GitHub API server-side and caches the count at the edge for
10 minutes. Set an optional secret so the Worker's requests are authenticated
(5000 req/hr instead of 60):

```sh
cd docs && pnpm exec wrangler secret put GITHUB_TOKEN
```

Use a fine-grained PAT with public repository read access only. Without the
secret the Worker still works, just at the unauthenticated rate limit. The
button hides the count entirely until the repo reaches 50 stars.
