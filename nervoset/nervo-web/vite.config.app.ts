import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

import { viteSingleFile } from "vite-plugin-singlefile"

// https://vitejs.dev/config/
export default defineConfig({
  base: 'https://nervo-web.metaelon.space/',

  define: {
    "process.env": {
      NODE_ENV: "production",
    },
  },

  plugins: [
    wasm(),
    topLevelAwait(),
    react(),
    viteSingleFile()
  ],

  build: {
    outDir: 'dist/app'
  }
});


