import { tester } from '#src/testing.ts';

import { requireSuppressionReasonRule } from './require-suppression-reason.ts';

tester.run('require-suppression-reason', requireSuppressionReasonRule, {
  valid: [
    {
      code: `// eslint-disable-next-line no-console -- legacy logger intentionally mirrors platform output
console.log('hello')
`,
      filename: 'src/example.ts',
    },
    {
      code: `/* oxlint-disable typescript/no-unsafe-assignment, no-console -- generated schema adapter narrows values later */
export const value = unknownValue
`,
      filename: 'src/example.ts',
    },
    {
      code: `// eslint-enable no-console
export const value = 1
`,
      filename: 'src/example.ts',
    },
  ],
  invalid: [
    {
      code: `// eslint-disable
export const value = 1
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'requireRuleIds' }],
    },
    {
      code: `/* oxlint-disable-next-line */
export const value = 1
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'requireRuleIds' }],
    },
    {
      code: `// eslint-disable-next-line no-console
console.log('hello')
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'requireReason' }],
    },
    {
      code: `// oxlint-disable-line typescript/no-explicit-any -- nope
const value: any = 1
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'requireLongerReason' }],
    },
  ],
});
