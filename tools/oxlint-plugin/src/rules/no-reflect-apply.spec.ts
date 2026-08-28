import { tester } from '#src/testing.ts';

import { noReflectApplyRule } from './no-reflect-apply.ts';

tester.run('no-reflect-apply', noReflectApplyRule, {
  valid: [
    {
      code: 'const result = handler(first, second);',
      filename: 'src/call.ts',
    },
    {
      code: 'function Reflect() { return { apply() {} }; }\nReflect().apply();',
      filename: 'src/shadowed.ts',
    },
  ],
  invalid: [
    {
      code: 'const result = Reflect.apply(handler, thisArg, args);',
      output: null,
      filename: 'src/call.ts',
      errors: [{ message: /typed function call/ }],
    },
    {
      code: 'const result = Reflect["apply"](handler, thisArg, args);',
      output: null,
      filename: 'src/call.ts',
      errors: [{ message: /dynamic dispatch/ }],
    },
  ],
});
