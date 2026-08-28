import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

const VIEWPORT_MEDIA_QUERY_PATTERN = /@media\b[^)]*\([^)]*\b(?:(?:min|max)-width|width)\b[^)]*\)/iu;

/** Require viewport media queries to come from a shared media contract, not local declarations. */
export const noLocalViewportMediaQueriesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description: 'Require viewport media queries to come from a shared media contract.',
    },
    messages: {
      useSharedViewportMediaQuery:
        'Do not declare a local viewport media query. Import a named query from the shared media contract.',
    },
  },
  create(context) {
    const reportViewportMediaQuery = (node: ESTree.Node) => {
      if (VIEWPORT_MEDIA_QUERY_PATTERN.test(context.sourceCode.getText(node))) {
        context.report({ node, messageId: 'useSharedViewportMediaQuery' });
      }
    };

    return {
      Literal: reportViewportMediaQuery,
      TemplateElement: reportViewportMediaQuery,
    };
  },
});
