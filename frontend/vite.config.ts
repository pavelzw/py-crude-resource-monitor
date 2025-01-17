import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile"

export default defineConfig(({ command, mode, ssrBuild }) => ({
    build: {
        minify: false,
    },
    plugins: [viteSingleFile()]
}));
