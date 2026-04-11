import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // Vite のサーバー設定（localhost のみ許可）
  server: {
    strictPort: true,
    host: "localhost",
  },
  // 静的アセットのパスを絶対パスで指定（Tauri 用）
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },
});
