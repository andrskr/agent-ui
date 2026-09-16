import { astryxStylex, LIGHTNINGCSS_TARGETS } from '@astryxdesign/build/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite-plus';

export default defineConfig({
  fmt: { singleQuote: true, printWidth: 100 },
  lint: {
    options: { typeAware: true, typeCheck: true },
  },
  plugins: [
    ...astryxStylex({
      rootDir: import.meta.dirname,
      lightningcssTargets: LIGHTNINGCSS_TARGETS,
    }),
    react(),
  ],
  server: { host: '127.0.0.1', port: 3100, strictPort: true },
  preview: { host: '127.0.0.1', port: 4100, strictPort: true },
});
