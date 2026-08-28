import { tester } from '#src/testing.ts';

import { requireSafetyCommentForTypeAssertionRule } from './require-safety-comment-for-type-assertion.ts';

tester.run('require-safety-comment-for-type-assertion', requireSafetyCommentForTypeAssertionRule, {
  valid: [
    {
      code: 'const frozen = [1, 2] as const;',
      filename: 'src/assert.ts',
    },
    {
      code: '// SAFETY: validated by the schema at the boundary\nconst value = input as Config;',
      filename: 'src/assert.ts',
    },
  ],
  invalid: [
    {
      code: 'const value = input as Config;',
      output: null,
      filename: 'src/assert.ts',
      errors: [{ message: /SAFETY:/ }],
    },
    {
      code: 'function read() { return input as Config; }',
      output: null,
      filename: 'src/assert.ts',
      errors: [{ message: /checked invariant/ }],
    },
  ],
});
