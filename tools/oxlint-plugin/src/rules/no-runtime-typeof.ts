import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

type RuntimeFunction = ESTree.ArrowFunctionExpression | ESTree.Function;

function isRuntimeFunction(node: ESTree.Node): node is RuntimeFunction {
  return (
    node.type === 'ArrowFunctionExpression' ||
    node.type === 'FunctionDeclaration' ||
    node.type === 'FunctionExpression'
  );
}

function isInsideTypeGuard(node: ESTree.Node): boolean {
  let current: ESTree.Node | null = node.parent;
  while (current !== null && current.type !== 'Program') {
    if (isRuntimeFunction(current)) {
      return current.returnType?.typeAnnotation.type === 'TSTypePredicate';
    }
    current = current.parent;
  }
  return false;
}

interface NoRuntimeTypeofOptions {
  allowInTypeGuards: boolean;
}

function readOptions(raw: unknown): NoRuntimeTypeofOptions {
  return {
    allowInTypeGuards:
      typeof raw === 'object' && raw !== null && 'allowInTypeGuards' in raw
        ? raw.allowInTypeGuards === true
        : false,
  };
}

/** Disallow runtime typeof checks that narrow unparsed values instead of decoding them. */
export const noRuntimeTypeofRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow runtime typeof checks; external values must be decoded into meaningful types at their I/O boundary.',
    },
    messages: {
      runtimeTypeof:
        'A `typeof` check narrows a representation without establishing its contract. Parse input at its I/O boundary, then branch on the domain value.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          allowInTypeGuards: { type: 'boolean' },
        },
        additionalProperties: false,
      },
    ],
  },
  create(context) {
    const { allowInTypeGuards } = readOptions(context.options[0]);

    return {
      UnaryExpression(node) {
        if (node.operator !== 'typeof') return;
        if (allowInTypeGuards && isInsideTypeGuard(node)) return;
        context.report({ node, messageId: 'runtimeTypeof' });
      },
    };
  },
});
