import { tester } from '#src/testing.ts';

import { noUnknownReturnsRule } from './no-unknown-returns.ts';

tester.run('no-unknown-returns', noUnknownReturnsRule, {
  valid: [
    {
      code: 'function load(): Config { return read(); }',
      filename: 'src/load.ts',
    },
    {
      code: 'function load(): Promise<Config> { return read(); }',
      filename: 'src/load.ts',
    },
  ],
  invalid: [
    {
      code: 'function load(): unknown { return read(); }',
      output: null,
      filename: 'src/load.ts',
      errors: [{ message: /to its caller/ }],
    },
    {
      code: 'function load(): Promise<unknown> { return read(); }',
      output: null,
      filename: 'src/load.ts',
      errors: [{ message: /named domain type/ }],
    },
  ],
});
