import { tester } from '#src/testing.ts';

import { noModuleMockingRule } from './no-module-mocking.ts';

tester.run('no-module-mocking', noModuleMockingRule, {
  valid: [
    {
      code: 'import { vi } from "vitest";\nconst spy = vi.fn();',
      filename: 'src/service.spec.ts',
    },
    {
      code: 'const store = { mock() {} };\nstore.mock();',
      filename: 'src/service.spec.ts',
    },
  ],
  invalid: [
    {
      code: 'import { vi } from "vitest";\nvi.mock("./dependency.ts");',
      output: null,
      filename: 'src/service.spec.ts',
      errors: [{ message: /dependency injection/ }],
    },
    {
      code: 'jest.mock("./dependency.ts");',
      output: null,
      filename: 'src/service.spec.ts',
      errors: [{ message: /real interface/ }],
    },
  ],
});
