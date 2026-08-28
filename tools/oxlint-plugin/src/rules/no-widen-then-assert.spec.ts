import { tester } from '#src/testing.ts';

import { noWidenThenAssertRule } from './no-widen-then-assert.ts';

tester.run('no-widen-then-assert', noWidenThenAssertRule, {
  valid: [
    {
      code: 'const config = { retries: 3 };\nconst typed = config as { retries: number };',
      filename: 'src/config.ts',
    },
    {
      code: 'const raw: unknown = fetchInput();\nuse(raw);',
      filename: 'src/config.ts',
    },
  ],
  invalid: [
    {
      code: 'const config: unknown = { retries: 3 };\nconst typed = config as { retries: number };',
      output: null,
      filename: 'src/config.ts',
      errors: [{ message: /discards type evidence and later recreates/ }],
    },
    {
      code: 'const config: object = { retries: 3 };\nconst typed = config as { retries: number };',
      output: null,
      filename: 'src/config.ts',
      errors: [{ message: /parse boundary input once/ }],
    },
  ],
});
