import { RuleTester } from 'oxlint/plugins-dev';
import { describe, it } from 'vite-plus/test';

RuleTester.describe = describe;
RuleTester.it = it;

export const tester = new RuleTester({
  languageOptions: {
    sourceType: 'module',
    parserOptions: {
      lang: 'tsx',
      ecmaFeatures: {
        jsx: true,
      },
    },
  },
});
