import { defineRule } from '@oxlint/plugins';

import { nodeName } from '#src/support/ast.ts';

/** Disallow the className and style escape hatches in favor of Astryx props and the xstyle prop. */
export const noStyleEscapeHatchesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow the className and style escape hatches. Astryx component props come first; the xstyle prop with StyleX tokens is the sanctioned fallback.',
    },
    messages: {
      noClassName:
        'Replace className with an Astryx component prop, or with the xstyle prop and StyleX tokens. Run `vp exec astryx docs styling` for the supported path.',
      noStyle:
        'Replace the style prop with an Astryx component prop, or with the xstyle prop and StyleX tokens. Run `vp exec astryx docs styling` for the supported path.',
    },
  },
  create(context) {
    return {
      JSXAttribute(node) {
        const name = nodeName(node.name);

        if (name === 'className') {
          context.report({ node, messageId: 'noClassName' });
          return;
        }

        if (name === 'style') {
          context.report({ node, messageId: 'noStyle' });
        }
      },
    };
  },
});
