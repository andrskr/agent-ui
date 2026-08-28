import { tester } from '#src/testing.ts';

import { noChainedTypeAssertionsRule } from './no-chained-type-assertions.ts';

tester.run('no-chained-type-assertions', noChainedTypeAssertionsRule, {
  valid: [
    {
      code: 'const value = input as Config;',
      filename: 'src/single.ts',
    },
    {
      code: 'const value = [1, 2] as const;',
      filename: 'src/const.ts',
    },
  ],
  invalid: [
    {
      code: 'const value = input as unknown as Config;',
      output: null,
      filename: 'src/chain.ts',
      errors: [{ message: /assertion chain discards type evidence/ }],
    },
    {
      code: 'const value = (input as unknown) as Config;',
      output: null,
      filename: 'src/paren-chain.ts',
      errors: [{ message: /parse untrusted input/ }],
    },
  ],
});
