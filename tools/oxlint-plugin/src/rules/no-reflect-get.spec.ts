import { tester } from '#src/testing.ts';

import { noReflectGetRule } from './no-reflect-get.ts';

tester.run('no-reflect-get', noReflectGetRule, {
  valid: [
    {
      code: 'const value = target.key;',
      filename: 'src/read.ts',
    },
    {
      code: 'function Reflect() { return {}; }\nconst value = Reflect().get;',
      filename: 'src/shadowed.ts',
    },
  ],
  invalid: [
    {
      code: 'const value = Reflect.get(target, "key");',
      output: null,
      filename: 'src/read.ts',
      errors: [{ message: /typed property access/ }],
    },
    {
      code: 'const value = Reflect["get"](target, "key");',
      output: null,
      filename: 'src/read.ts',
      errors: [{ message: /named domain type/ }],
    },
  ],
});
