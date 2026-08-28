import { tester } from '#src/testing.ts';

import { noEffectDiagnosticDirectivesRule } from './no-effect-diagnostic-directives.ts';

tester.run('no-effect-diagnostic-directives', noEffectDiagnosticDirectivesRule, {
  valid: [
    {
      code: `// Effect diagnostics are enforced by the repository Oxlint gate.
export const value = 1
`,
      filename: 'src/example.ts',
    },
  ],
  invalid: [
    {
      code: `// @effect-diagnostics floatingEffect:off
export const value = 1
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'forbidden' }],
    },
    {
      code: `/* @effect-diagnostics-next-line floatingEffect:off */
export const value = 1
`,
      filename: 'src/example.ts',
      errors: [{ messageId: 'forbidden' }],
    },
  ],
});
