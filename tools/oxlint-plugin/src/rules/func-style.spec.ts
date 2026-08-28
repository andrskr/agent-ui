import { tester } from '#src/testing.ts';

import { funcStyleRule } from './func-style.ts';

tester.run('func-style', funcStyleRule, {
  valid: [
    {
      code: 'function nodeName(node) { return node.name; }',
      filename: 'src/example.ts',
      options: ['declaration'],
    },
    {
      code: 'const getName: NameReader = (node) => node.name;',
      filename: 'src/example.ts',
      options: ['declaration', { allowTypeAnnotation: true }],
    },
    {
      code: 'const read = () => this.value;',
      filename: 'src/example.ts',
      options: ['declaration'],
    },
    {
      code: 'const read = () => node.name;',
      filename: 'src/example.ts',
      options: ['declaration', { allowArrowFunctions: true }],
    },
    {
      code: 'export const read = (node) => node.name;',
      filename: 'src/example.ts',
      options: ['declaration', { overrides: { namedExports: 'ignore' } }],
    },
    {
      code: 'export const read = (node) => node.name;',
      filename: 'src/example.ts',
      options: ['declaration', { overrides: { namedExports: 'expression' } }],
    },
    {
      code: 'function overload(value: string): string; function overload(value) { return value; }',
      filename: 'src/example.ts',
      options: ['expression'],
    },
  ],
  invalid: [
    {
      code: 'const nodeName = (node) => node.name;',
      output: `function nodeName(node) {
  return node.name;
}`,
      filename: 'src/example.ts',
      options: ['declaration'],
      errors: [{ messageId: 'declaration' }],
    },
    {
      code: 'const read = function (node) { return node.name; };',
      output: 'function read(node) { return node.name; }',
      filename: 'src/example.ts',
      options: ['declaration'],
      errors: [{ messageId: 'declaration' }],
    },
    {
      code: 'export const read = (node) => { return node.name; };',
      output: 'export function read(node) { return node.name; }',
      filename: 'src/example.ts',
      options: ['declaration'],
      errors: [{ messageId: 'declaration' }],
    },
    {
      code: 'function read(node) { return node.name; }',
      filename: 'src/example.ts',
      options: ['expression'],
      errors: [{ messageId: 'expression' }],
    },
    {
      code: 'export const read = (node) => node.name;',
      output: `export function read(node) {
  return node.name;
}`,
      filename: 'src/example.ts',
      options: ['expression', { overrides: { namedExports: 'declaration' } }],
      errors: [{ messageId: 'declaration' }],
    },
    {
      code: 'const first = () => 1, second = () => 2;',
      output: null,
      filename: 'src/example.ts',
      options: ['declaration'],
      errors: [{ messageId: 'declaration' }, { messageId: 'declaration' }],
    },
  ],
});
