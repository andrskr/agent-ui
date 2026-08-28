import { tester } from '#src/testing.ts';

import { noConditionalEmptyObjectSpreadRule } from './no-conditional-empty-object-spread.ts';

tester.run('no-conditional-empty-object-spread', noConditionalEmptyObjectSpreadRule, {
  valid: [
    {
      code: 'const merged = { ...base };',
      filename: 'src/merge.ts',
    },
    {
      code: 'const merged = { ...(flag ? first : second) };',
      filename: 'src/merge.ts',
    },
  ],
  invalid: [
    {
      code: 'const merged = { ...(flag ? { active: true } : {}) };',
      output: null,
      filename: 'src/merge.ts',
      errors: [{ message: /conditional spread hides property omission/ }],
    },
    {
      code: 'const merged = { ...(flag ? {} : { active: true }) };',
      output: null,
      filename: 'src/merge.ts',
      errors: [{ message: /separate statements/ }],
    },
  ],
});
