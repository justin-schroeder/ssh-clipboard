import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

const allowedHosts = ['macbookserver.tail13bd39.ts.net', '.ts.net']

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  server: { allowedHosts },
  preview: { allowedHosts },
})
