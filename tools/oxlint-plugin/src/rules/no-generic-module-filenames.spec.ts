import { tester } from '#src/testing.ts';

import { noGenericModuleFilenamesRule } from './no-generic-module-filenames.ts';

tester.run('no-generic-module-filenames', noGenericModuleFilenamesRule, {
  valid: [
    {
      code: 'export const value = 1',
      filename: 'tools/oxlint-plugin/src/ast-nodes.ts',
    },
    {
      code: 'export const rules = {}',
      filename: 'tools/oxlint-plugin/src/index.ts',
    },
  ],
  invalid: [
    {
      code: 'export const value = 1',
      filename: 'src/utils.ts',
      errors: [{ messageId: 'genericFilename' }],
    },
    {
      code: 'export type Value = string',
      filename: 'src/components/types.ts',
      errors: [{ messageId: 'genericFilename' }],
    },
  ],
});
