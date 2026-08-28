import { tester } from '#src/testing.ts';

import { noVitestTestLifecycleCallbacksRule } from './no-vitest-test-lifecycle-callbacks.ts';

tester.run('no-vitest-test-lifecycle-callbacks', noVitestTestLifecycleCallbacksRule, {
  valid: [
    {
      code: `
import { afterAll, afterEach, beforeAll, beforeEach } from 'vitest'
beforeAll(setup)
beforeEach(reset)
afterEach(restore)
afterAll(teardown)
`,
      filename: 'example.spec.ts',
    },
    {
      code: `
import { test as baseTest } from 'vitest'
const test = baseTest.extend('resource', ({ task: _task }, { onCleanup }) => {
  const resource = acquireResource()
  onCleanup(() => resource.dispose())
  return resource
})
test('works', ({ resource }) => resource.use())
`,
      filename: 'example.spec.ts',
    },
    {
      code: `
const onTestFinished = createDomainCallback()
onTestFinished()
`,
      filename: 'example.spec.ts',
    },
  ],
  invalid: [
    {
      code: `
import { onTestFailed, onTestFinished } from 'vitest'
onTestFailed(captureDiagnostics)
onTestFinished(dispose)
`,
      filename: 'example.spec.ts',
      errors: [
        { message: /onTestFailed\(\) is banned/ },
        { message: /onTestFinished\(\) is banned/ },
      ],
    },
    {
      code: `
import { onTestFinished as cleanup } from '@effect/vitest'
cleanup(dispose)
`,
      filename: 'example.spec.ts',
      errors: [{ message: /cleanup\(\) is banned/ }],
    },
    {
      code: `
import * as vitest from 'vite-plus/test'
vitest.onTestFailed(captureDiagnostics)
vitest.onTestFinished(dispose)
`,
      filename: 'example.spec.ts',
      errors: [
        { message: /vitest\.onTestFailed\(\) is banned/ },
        { message: /vitest\.onTestFinished\(\) is banned/ },
      ],
    },
  ],
});
