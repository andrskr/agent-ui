import { tester } from '#src/testing.ts';

import { preferArrowCallbackRule } from './prefer-arrow-callback.ts';

tester.run('prefer-arrow-callback', preferArrowCallbackRule, {
  valid: [
    {
      code: 'items.map((item) => item.id);',
      filename: 'src/example.ts',
    },
    {
      code: 'items.map(function named(item) { return item.id; });',
      filename: 'src/example.ts',
      options: [{ allowNamedFunctions: true }],
    },
    {
      code: 'items.map(function walk(item) { return item.children.map(walk); });',
      filename: 'src/example.ts',
    },
    {
      code: 'items.map(function (item) { return arguments[0] ?? item; });',
      filename: 'src/example.ts',
    },
    {
      code: 'items.map(function (item) { return this.read(item); });',
      filename: 'src/example.ts',
    },
    {
      code: 'items.map(function () { return new.target; });',
      filename: 'src/example.ts',
    },
    {
      code: 'items.map(function* (item) { yield item; });',
      filename: 'src/example.ts',
    },
  ],
  invalid: [
    {
      code: 'items.map(function (item) { return item.id; });',
      output: null,
      filename: 'src/example.ts',
      errors: [{ messageId: 'preferArrowCallback' }],
    },
    {
      code: 'items.map(function named(item) { return item.id; });',
      output: null,
      filename: 'src/example.ts',
      errors: [{ messageId: 'preferArrowCallback' }],
    },
    {
      code: 'items.map(async function (item) { return item.id; });',
      output: null,
      filename: 'src/example.ts',
      errors: [{ messageId: 'preferArrowCallback' }],
    },
    {
      code: 'items.map(function () { return this.value; }.bind(this));',
      output: null,
      filename: 'src/example.ts',
      errors: [{ messageId: 'preferArrowCallback' }],
    },
    {
      code: 'items.map(function () { return this.value; });',
      output: null,
      filename: 'src/example.ts',
      options: [{ allowUnboundThis: false }],
      errors: [{ messageId: 'preferArrowCallback' }],
    },
    {
      code: 'items.map(function () { return this.value; }.bind(/* stable receiver */ this));',
      output: null,
      filename: 'src/example.ts',
      errors: [{ messageId: 'preferArrowCallback' }],
    },
  ],
});
