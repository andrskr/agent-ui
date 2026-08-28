import { tester } from '#src/testing.ts';

import { noUnknownParametersRule } from './no-unknown-parameters.ts';

tester.run('no-unknown-parameters', noUnknownParametersRule, {
  valid: [
    {
      code: 'function handle(event: DomainEvent) { return event; }',
      filename: 'src/handle.ts',
    },
    {
      code: 'function wrap(cause: unknown) { throw new Error("failed", { cause }); }',
      filename: 'src/wrap.ts',
    },
  ],
  invalid: [
    {
      code: 'function handle(event: unknown) { return event; }',
      output: null,
      filename: 'src/handle.ts',
      errors: [{ message: /leaves input unparsed/ }],
    },
    {
      code: 'const handle = (value: unknown) => value;',
      output: null,
      filename: 'src/handle.ts',
      errors: [{ message: /I\/O boundary/ }],
    },
  ],
});
