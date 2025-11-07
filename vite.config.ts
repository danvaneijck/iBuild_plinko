import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { nodePolyfills } from '@bangjelkoski/vite-plugin-node-polyfills'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), nodePolyfills()],
  optimizeDeps: {
    exclude: ['lucide-react'],
  },
})
