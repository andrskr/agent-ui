import { tester } from '#src/testing.ts';

import { noRawLayoutElementsRule } from './no-raw-layout-elements.ts';

tester.run('no-raw-layout-elements', noRawLayoutElementsRule, {
  valid: [
    {
      code: `
import { Card } from '@astryxdesign/core/Card';
import { Grid, HStack, Layout, LayoutContent, VStack } from '@astryxdesign/core/Layout';
import { Section } from '@astryxdesign/core/Section';

export function Page() {
  return (
    <VStack as="main" gap={4}>
      <Section>
        <Grid columns={{ minWidth: 240, repeat: 'fit' }}>
          <Card />
          <HStack />
        </Grid>
      </Section>
      <Layout content={<LayoutContent />} />
    </VStack>
  );
}
`,
      filename: 'apps/website/src/page.tsx',
    },
    {
      code: `
export function RootDocument() {
  return (
    <html lang="en">
      <head>
        <link rel="icon" href="/favicon.svg" />
      </head>
      <body />
    </html>
  );
}
`,
      filename: 'apps/website/src/root-document.tsx',
    },
  ],
  invalid: [
    {
      code: `
export function Page() {
  return <div />;
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [{ message: /Raw <div> bypasses Astryx layout composition/ }],
    },
    {
      code: `
export function Page() {
  return (
    <main>
      <section>
        <nav />
        <ul>
          <li>Item</li>
        </ul>
      </section>
    </main>
  );
}
`,
      filename: 'apps/website/src/page.tsx',
      errors: [
        { message: /Raw <main> bypasses Astryx layout composition/ },
        { message: /Raw <section> bypasses Astryx layout composition/ },
        { message: /Raw <nav> bypasses Astryx layout composition/ },
        { message: /Raw <ul> bypasses Astryx layout composition/ },
        { message: /Raw <li> bypasses Astryx layout composition/ },
      ],
    },
  ],
});
