import { defineConfig } from 'vite-plus';

import { qualityConfig } from './tools/quality/config.ts';

export default defineConfig({
  ...qualityConfig(),
  run: {
    cache: true,
    tasks: {
      'agent-ui': {
        command: 'cargo run -p agent-ui --',
        cache: false,
      },
      'verify:agent-ui': {
        command:
          'cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -p agent-ui -- tasks --check',
        cache: false,
      },
    },
  },
});
