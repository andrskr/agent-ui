import { tester } from '#src/testing.ts';

import { noTestContainersRule } from './no-test-containers.ts';

tester.run('no-test-containers', noTestContainersRule, {
  valid: [
    {
      code: `
import { test } from 'vitest'
test('works', () => {})
`,
      filename: 'example.spec.ts',
    },
    {
      code: `
const describe = createNarrative()
describe('not a test container')
`,
      filename: 'example.spec.ts',
    },
  ],
  invalid: [
    {
      code: `
import { describe, suite } from 'vitest'
describe('group', () => {})
suite.skip('group', () => {})
`,
      filename: 'example.spec.ts',
      errors: [{ message: /describe\(\) is banned/ }, { message: /suite\.skip\(\) is banned/ }],
    },
    {
      code: `
import { describe as group } from 'vitest'
group.only('group', () => {})
`,
      filename: 'example.spec.ts',
      errors: [{ message: /group\.only\(\) is banned/ }],
    },
    {
      code: `
import * as vitest from 'vitest'
vitest.describe('group', () => {})
`,
      filename: 'example.spec.ts',
      errors: [{ message: /vitest\.describe\(\) is banned/ }],
    },
    {
      code: `
import { describe } from 'vite-plus/test'
describe('group', () => {})
`,
      filename: 'example.spec.ts',
      errors: [{ message: /describe\(\) is banned/ }],
    },
    {
      code: `
import { describe } from '@effect/vitest'
describe('group', () => {})
`,
      filename: 'example.spec.ts',
      errors: [{ message: /describe\(\) is banned/ }],
    },
  ],
});
