import { LIGHTNINGCSS_TARGETS, astryxStylex } from '@astryxdesign/build/vite';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import viteReact from '@vitejs/plugin-react';
import { defineConfig } from 'vite-plus';
import type { Plugin } from 'vite-plus';

const astryxCorePackage = '@astryxdesign/core';

function configureAstryxSsr(): Plugin {
  return {
    name: 'web:configure-astryx-ssr',
    enforce: 'post',
    configResolved(config) {
      const existingExcludes = config.environments.ssr.optimizeDeps.exclude ?? [];
      config.environments.ssr.optimizeDeps.exclude = [
        ...new Set([...existingExcludes, astryxCorePackage]),
      ];
    },
  };
}

export default defineConfig({
  resolve: { tsconfigPaths: true },
  plugins: [
    tanstackStart(),
    ...astryxStylex({
      rootDir: import.meta.dirname,
      lightningcssTargets: LIGHTNINGCSS_TARGETS,
    }),
    configureAstryxSsr(),
    viteReact({ compiler: true }),
  ],
});
