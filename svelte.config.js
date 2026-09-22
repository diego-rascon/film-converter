// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),

    // The routes live in `src/app`, beside the other top-level folders, rather
    // than in SvelteKit's default `src/routes`.
    files: {
      routes: "src/app",
    },

    // One alias per top-level folder, so an import says which layer it crosses
    // into. `$lib` is SvelteKit's own and keeps its default, `src/lib`.
    // Inside a feature, siblings are imported relatively.
    alias: {
      $components: "src/components",
      $features: "src/features",
      $hooks: "src/hooks",
      $state: "src/state",
      $styles: "src/styles",
      $types: "src/types",
      $utils: "src/utils",
    },
  },
};

export default config;
