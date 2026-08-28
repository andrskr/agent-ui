import { tester } from '#src/testing.ts';

import { noRuntimeTypeofRule } from './no-runtime-typeof.ts';

tester.run('no-runtime-typeof', noRuntimeTypeofRule, {
  valid: [
    {
      code: 'const kind = descriptor.kind;',
      filename: 'src/kind.ts',
    },
    {
      code: 'function isText(value: unknown): value is string { return typeof value === "string"; }',
      options: [{ allowInTypeGuards: true }],
      filename: 'src/guard.ts',
    },
  ],
  invalid: [
    {
      code: 'const kind = typeof value;',
      output: null,
      filename: 'src/kind.ts',
      errors: [{ message: /narrows a representation/ }],
    },
    {
      code: 'if (typeof value === "string") { use(value); }',
      output: null,
      filename: 'src/kind.ts',
      errors: [{ message: /I\/O boundary/ }],
    },
  ],
});
