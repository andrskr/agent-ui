import { tester } from '#src/testing.ts';

import { noStyleEscapeHatchesRule } from './no-style-escape-hatches.ts';

tester.run('no-style-escape-hatches', noStyleEscapeHatchesRule, {
  valid: [
    {
      code: `
import { Card } from '@astryxdesign/core/Card';
import { VStack } from '@astryxdesign/core/Layout';

export function Page() {
  return <VStack gap={4}><Card padding={5} /></VStack>;
}
`,
      filename: 'apps/website/src/page.tsx',
    },
    {
      code: `
export function Page({ styles }) {
  return <Component xstyle={styles.root} />;
}
`,
      filename: 'apps/website/src/xstyle.tsx',
    },
  ],
  invalid: [
    {
      code: `
export function Page() {
  return <Component className="surface" />;
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [{ messageId: 'noClassName' }],
    },
    {
      code: `
export function Page() {
  return <Component style={{ color: '#fff' }} />;
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [{ messageId: 'noStyle' }],
    },
    {
      code: `
export function Page() {
  return <Component style={{ padding: '16px' }} />;
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [{ messageId: 'noStyle' }],
    },
    {
      code: `
export function TokenBackedStyle() {
  return <Component style={{ padding: 'var(--spacing-4)' }} />;
}
`,
      filename: 'apps/website/src/token-style.tsx',
      errors: [{ messageId: 'noStyle' }],
    },
    {
      code: `
const rowPadding = { paddingBlock: 'var(--spacing-4)' };

export function HoistedStyle() {
  return <Component style={rowPadding} />;
}
`,
      filename: 'apps/website/src/hoisted-style.tsx',
      errors: [{ messageId: 'noStyle' }],
    },
    {
      code: `
export function Page({ style }) {
  return <Component style={{ ...style }} />;
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [{ messageId: 'noStyle' }],
    },
  ],
});
