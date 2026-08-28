import { tester } from '#src/testing.ts';

import { noUnsafeDictionaryTypeRule } from './no-unsafe-dictionary-type.ts';

tester.run('no-unsafe-dictionary-type', noUnsafeDictionaryTypeRule, {
  valid: [
    {
      code: 'type Scores = Record<string, number>;',
      filename: 'src/scores.ts',
    },
    {
      code: 'type Config = { name: string };',
      filename: 'src/config.ts',
    },
  ],
  invalid: [
    {
      code: 'type Loose = Record<string, any>;',
      output: null,
      filename: 'src/loose.ts',
      errors: [{ message: /no concrete value contract/ }],
    },
    {
      code: 'type Bag = Record<string, unknown>;',
      output: null,
      filename: 'src/bag.ts',
      errors: [{ message: /parse external payloads/ }],
    },
  ],
});
