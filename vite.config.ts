import { defineConfig } from 'vite'

export default defineConfig({
  root: 'src',           // tu carpeta de frontend
  clearScreen: false,    // para ver logs de Tauri en la misma terminal
  server: {
    port: 1420,          // puerto que Tauri espera por defecto
    strictPort: true,
  },
  build: {
    target: 'esnext',
    outDir: '../dist',   // Tauri leerá de aquí
    emptyOutDir: true,
  },
})