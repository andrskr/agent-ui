import { tester } from '#src/testing.ts';

import { noKnownValueWideningRule } from './no-known-value-widening.ts';

tester.run('no-known-value-widening', noKnownValueWideningRule, {
  valid: [
    {
      code: 'const config = { retries: 3 };',
      filename: 'src/config.ts',
    },
    {
      code: 'type Config = { retries: number };\nconst config: Config = { retries: 3 };',
      filename: 'src/config.ts',
    },
  ],
  invalid: [
    {
      code: 'const config: object = { retries: 3 };',
      output: null,
      filename: 'src/config.ts',
      errors: [{ message: /discards known type evidence/ }],
    },
    {
      code: 'const label: unknown = "ready";',
      output: null,
      filename: 'src/config.ts',
      errors: [{ message: /named owner contract/ }],
    },
  ],
});
