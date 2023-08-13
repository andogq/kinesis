import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
    plugins: [wasmPack("kinesis")],
    server: {
        watch: {
            // This should work, but it doesn't https://vitejs.dev/config/server-options.html#server-watch
            ignored: [
                "!**/kinesis/**",
            ],
        },
    },
    optimizeDeps: {
        exclude: [
            "kinesis",
        ],
    },
});
