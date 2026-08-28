import { tester } from '#src/testing.ts';

import { noUnknownTypeAliasesRule } from './no-unknown-type-aliases.ts';

tester.run('no-unknown-type-aliases', noUnknownTypeAliasesRule, {
  valid: [
    {
      code: 'type UserId = string;',
      filename: 'src/types.ts',
    },
    {
      code: 'type Maybe<T> = T | null;',
      filename: 'src/types.ts',
    },
  ],
  invalid: [
    {
      code: 'type Anything = unknown;',
      output: null,
      filename: 'src/types.ts',
      errors: [{ message: /hides .unknown./ }],
    },
    {
      code: 'type Raw = unknown;\ntype Payload = Raw;',
      output: null,
      filename: 'src/types.ts',
      errors: [{ message: /hides .unknown./ }, { message: /hides .unknown./ }],
    },
  ],
});
