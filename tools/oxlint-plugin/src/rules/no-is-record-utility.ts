import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

import { importedName } from '#src/support/ast.ts';

const BANNED_NAME = 'isRecord';

/** Ban generic isRecord guards; they prove only object-ness, not the fields code relies on. */
export const noIsRecordUtilityRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow generic isRecord checks because they do not validate the fields used by code.',
    },
    messages: {
      noIsRecord:
        'isRecord is banned. A generic record check proves only that a value is an object; it does not validate the fields that code uses. Decode unknown input with Effect Schema at its boundary. For trusted internal data, write a guard for the exact local shape.',
    },
  },
  create(context) {
    const reportBannedName = (node: ESTree.Node | null | undefined) => {
      if (node?.type === 'Identifier' && node.name === BANNED_NAME) {
        context.report({ node, messageId: 'noIsRecord' });
      }
    };

    return {
      FunctionDeclaration(node) {
        reportBannedName(node.id);
      },
      FunctionExpression(node) {
        reportBannedName(node.id);
      },
      TSDeclareFunction(node) {
        reportBannedName(node.id);
      },
      VariableDeclarator(node) {
        reportBannedName(node.id);
      },
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) {
          const imported = specifier.type === 'ImportSpecifier' ? importedName(specifier) : null;
          if (specifier.local.name === BANNED_NAME || imported === BANNED_NAME) {
            context.report({ node: specifier, messageId: 'noIsRecord' });
          }
        }
      },
    };
  },
});
