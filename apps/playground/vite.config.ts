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

function configureAstryxStylex(): Array<Plugin> {
  const plugins = astryxStylex({
    rootDir: import.meta.dirname,
    lightningcssTargets: LIGHTNINGCSS_TARGETS,
  });
  for (const plugin of plugins) {
    if (plugin.name === 'astryx-build-layer-split') {
      // The client build emits CSS. The server build does not.
      plugin.applyToEnvironment = (environment) => environment.name === 'client';
    }
  }
  return plugins;
}

export default defineConfig({
  resolve: { tsconfigPaths: true },
  plugins: [
    tanstackStart(),
    ...configureAstryxStylex(),
    configureAstryxSsr(),
    viteReact({ compiler: true }),
  ],
});
