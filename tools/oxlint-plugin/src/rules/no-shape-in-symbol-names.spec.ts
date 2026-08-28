import { tester } from '#src/testing.ts';

import { noForbiddenTermInSymbolNamesRule } from './no-shape-in-symbol-names.ts';

tester.run('no-shape-in-symbol-names', noForbiddenTermInSymbolNamesRule, {
  valid: [
    {
      code: 'const boundingBox = { width: 1, height: 2 };',
      filename: 'src/geometry.ts',
    },
    {
      code: 'function renderChart() { return null; }',
      filename: 'src/chart.ts',
    },
  ],
  invalid: [
    {
      code: 'const shapeData = { width: 1 };',
      output: null,
      filename: 'src/geometry.ts',
      errors: [{ message: /domain role/ }],
    },
    {
      code: 'function getShape() { return null; }',
      output: null,
      filename: 'src/geometry.ts',
      errors: [{ message: /describes structure rather than ownership/ }],
    },
  ],
});
