import { defineRule } from '@oxlint/plugins';

import { nodeName } from '#src/support/ast.ts';

const RAW_LAYOUT_ELEMENTS = new Set([
  'article',
  'aside',
  'div',
  'footer',
  'header',
  'li',
  'main',
  'nav',
  'ol',
  'section',
  'ul',
]);

const ELEMENT_HINTS: Record<string, string> = {
  article: 'Use Section, Card, LayoutContent, or another Astryx region component.',
  aside: 'Use LayoutPanel, SideNav, or another Astryx panel/navigation component.',
  div: 'Use VStack, HStack, Grid, Layout, Section, Card, List/Item, or Table.',
  footer: 'Use LayoutFooter or an Astryx layout component with as="footer".',
  header: 'Use LayoutHeader or an Astryx layout component with as="header".',
  li: 'Use List/Item or Table for scannable rows.',
  main: 'Use AppShell, LayoutContent, or an Astryx layout component with as="main".',
  nav: 'Use SideNav, AppShell navigation slots, or an Astryx layout component with as="nav".',
  ol: 'Use List/Item or Table for scannable rows.',
  section: 'Use Section, Card, LayoutContent, or an Astryx layout component with as="section".',
  ul: 'Use List/Item or Table for scannable rows.',
};

/** Disallow raw JSX layout elements so application UI composes through Astryx components. */
export const noRawLayoutElementsRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow raw JSX layout elements so application UI composes through Astryx components.',
    },
    messages: {
      noRawLayoutElement:
        'Raw <{{element}}> bypasses Astryx layout composition. {{hint}} If you are unsure which component fits, run `vp exec astryx build "<intent>"` before writing UI.',
    },
  },
  create(context) {
    return {
      JSXOpeningElement(node) {
        const element = nodeName(node.name);
        if (element === null || !RAW_LAYOUT_ELEMENTS.has(element)) return;

        context.report({
          node,
          messageId: 'noRawLayoutElement',
          data: {
            element,
            hint: ELEMENT_HINTS[element] ?? 'Use Astryx components instead of raw layout tags.',
          },
        });
      },
    };
  },
});
