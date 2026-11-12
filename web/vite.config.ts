import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const packages = resolve(here, "../../adamo-web/adamo-ts/packages");

export default defineConfig({
  plugins: [react(), tailwindcss(), wasm()],
  resolve: {
    alias: {
      "adamo/crypto": resolve(packages, "adamo/src/crypto/index.ts"),
      "adamo/transport": resolve(packages, "adamo/src/transport/index.ts"),
      "adamo/ros": resolve(packages, "adamo/src/ros/index.ts"),
      "adamo/media": resolve(packages, "adamo/src/media/index.ts"),
      "adamo/teleop": resolve(packages, "adamo/src/teleop/index.ts"),
      "adamo/fleet": resolve(packages, "adamo/src/fleet/index.ts"),
      "adamo-react": resolve(packages, "adamo-react/src/index.tsx"),
      "@adamo-tech/zenoh-webtransport": resolve(packages, "zenoh-webtransport/src/index.ts"),
    },
    dedupe: ["react", "react-dom"],
  },
  server: {
    port: 5180,
    fs: { allow: [here, packages, resolve(packages, "../node_modules")] },
  },
});
