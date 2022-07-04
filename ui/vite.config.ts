import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import externalGlobals from "rollup-plugin-external-globals";


// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    externalGlobals({
      react: "React"
    })],
  envDir: '../environments',
  envPrefix: 'REA_PLAYSPACE_',
  server: {
      port: 8080
  },
  build: {
    rollupOptions: {
      external: [
        'react'
      ],
    }
  }
})
