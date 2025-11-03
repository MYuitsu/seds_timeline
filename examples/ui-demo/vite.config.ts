import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  root: __dirname,
  server: {
    port: 5173,
    fs: {
      allow: [
        __dirname,
        resolve(__dirname, "..", "..", "pkg")
      ]
    }
  }
});
