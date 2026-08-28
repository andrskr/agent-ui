import { tester } from '#src/testing.ts';

import { noLocalViewportMediaQueriesRule } from './no-local-viewport-media-queries.ts';

tester.run('no-local-viewport-media-queries', noLocalViewportMediaQueriesRule, {
  valid: [
    {
      code: `
const reducedMotion = '@media (prefers-reduced-motion: reduce)';
`,
      filename: 'src/preferences/motion.ts',
    },
  ],
  invalid: [
    {
      code: `
const desktop = '@media (min-width: 900px)';
`,
      filename: 'src/privacy-policy-page.tsx',
      errors: [{ messageId: 'useSharedViewportMediaQuery' }],
    },
    {
      code: `
const compact = '@media (max-width: 48rem)';
`,
      filename: 'src/privacy-policy-page.tsx',
      errors: [{ messageId: 'useSharedViewportMediaQuery' }],
    },
    {
      code: `
const desktop = \`@media (width >= 1024px)\`;
`,
      filename: 'src/privacy-policy-page.tsx',
      errors: [{ messageId: 'useSharedViewportMediaQuery' }],
    },
    {
      code: `
const desktop = '@media (64rem <= width)';
`,
      filename: 'src/privacy-policy-page.tsx',
      errors: [{ messageId: 'useSharedViewportMediaQuery' }],
    },
  ],
});
