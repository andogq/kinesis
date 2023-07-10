import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
    plugins: [wasmPack("framework")],
    server: {
        watch: {
            // This should work, but it doesn't https://vitejs.dev/config/server-options.html#server-watch
            ignored: [
                "!**/framework/**",
            ],
        },
    },
    optimizeDeps: {
        exclude: [
            "framework",
        ],
    },
});
