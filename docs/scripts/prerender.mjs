// Renders the Vue app to static HTML after `vite build`.
// Run via `npm run build`; expects dist/ (client) and dist-ssr/ (server bundle).
import { readFile, writeFile, rm } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

const dist = (p) => fileURLToPath(new URL(`../dist/${p}`, import.meta.url))

const { render } = await import(new URL('../dist-ssr/entry-server.js', import.meta.url))

const template = await readFile(dist('index.html'), 'utf8')
const appHtml = await render()

if (!template.includes('<!--app-html-->')) {
  throw new Error('dist/index.html is missing the <!--app-html--> outlet')
}

await writeFile(dist('index.html'), template.replace('<!--app-html-->', appHtml))
await rm(fileURLToPath(new URL('../dist-ssr', import.meta.url)), { recursive: true, force: true })

console.log('prerendered dist/index.html')
