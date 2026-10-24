import { resolve } from "node:path";
import { defineConfig } from "vite";

// Deux pages : la fenêtre principale et la pastille du haut de l'écran.
export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: "es2021",
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        island: resolve(__dirname, "island.html"),
      },
    },
  },
});
