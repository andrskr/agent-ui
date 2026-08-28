import { tester } from '#src/testing.ts';

import { noIsRecordUtilityRule } from './no-is-record-utility.ts';

tester.run('no-is-record-utility', noIsRecordUtilityRule, {
  valid: [
    {
      code: 'function isFeatureAttributes(value: unknown) { return value !== null }',
      filename: 'src/feature-attributes.ts',
    },
    {
      code: 'const parser = { isRecord() { return true } }',
      filename: 'src/parser.ts',
    },
  ],
  invalid: [
    {
      code: 'function isRecord(value: unknown) { return typeof value === "object" }',
      output: null,
      filename: 'src/record.ts',
      errors: [{ message: /Effect Schema/ }],
    },
    {
      code: 'const isRecord = (value: unknown) => value !== null',
      output: null,
      filename: 'src/record.ts',
      errors: [{ message: /exact local shape/ }],
    },
    {
      code: 'declare function isRecord(value: unknown): boolean',
      output: null,
      filename: 'src/record.d.ts',
      errors: [{ message: /does not validate the fields/ }],
    },
    {
      code: 'import isRecord from "./record.ts"',
      output: null,
      filename: 'src/use-record.ts',
      errors: [{ message: /Effect Schema/ }],
    },
    {
      code: 'import { isRecord as hasObjectShape } from "./record.ts"',
      output: null,
      filename: 'src/use-record.ts',
      errors: [{ message: /exact local shape/ }],
    },
    {
      code: 'import { isObject as isRecord } from "./record.ts"',
      output: null,
      filename: 'src/use-record.ts',
      errors: [{ message: /does not validate the fields/ }],
    },
  ],
});
