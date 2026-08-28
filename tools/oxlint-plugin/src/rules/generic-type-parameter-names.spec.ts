import { tester } from '#src/testing.ts';

import { genericTypeParameterNamesRule } from './generic-type-parameter-names.ts';

tester.run('generic-type-parameter-names', genericTypeParameterNamesRule, {
  valid: [
    {
      code: 'interface Box<TValue> { value: TValue }',
      filename: 'src/box.ts',
    },
    {
      code: 'type Result<TValue, TError> = { value: TValue } | { error: TError }',
      filename: 'src/result.ts',
    },
  ],
  invalid: [
    {
      code: 'type Box<T> = T',
      output: null,
      filename: 'src/box.ts',
      errors: [{ message: /TValue/ }],
    },
    {
      code: 'interface Dict<K, V> { key: K; value: V }',
      output: null,
      filename: 'src/dict.ts',
      errors: [{ message: /TKey/ }, { message: /TValue/ }],
    },
    {
      code: 'interface Loader<Data> { load(): Data }',
      output: null,
      filename: 'src/loader.ts',
      errors: [{ message: /TData/ }],
    },
    {
      code: 'interface Box<Tvalue> { value: Tvalue }',
      output: null,
      filename: 'src/box.ts',
      errors: [{ message: /TValue/ }],
    },
    {
      code: 'interface Mapper<X> { map(value: X): X }',
      output: null,
      filename: 'src/mapper.ts',
      errors: [{ message: /TX/ }],
    },
    {
      code: `
type Box<T> = T
type Pair<T> = T
`,
      output: null,
      filename: 'src/box.ts',
      errors: [{ message: /TValue/ }, { message: /TValue/ }],
    },
  ],
});
