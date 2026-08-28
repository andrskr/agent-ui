import { tester } from '#src/testing.ts';

import { noObjectParametersRule } from './no-object-parameters.ts';

tester.run('no-object-parameters', noObjectParametersRule, {
  valid: [
    {
      code: 'function save(record: UserRecord) { return record; }',
      filename: 'src/save.ts',
    },
    {
      code: 'function save(record: { id: string }) { return record; }',
      filename: 'src/save.ts',
    },
  ],
  invalid: [
    {
      code: 'function save(record: object) { return record; }',
      output: null,
      filename: 'src/save.ts',
      errors: [{ message: /uses the broad .object. type/ }],
    },
    {
      code: 'const save = (record: object) => record;',
      output: null,
      filename: 'src/save.ts',
      errors: [{ message: /named owner type/ }],
    },
  ],
});
