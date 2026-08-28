import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

import { identifierName, nodeName } from '#src/support/ast.ts';
import { collectTestFrameworkImport } from '#src/support/test-framework-imports.ts';

const BANNED_TEST_LIFECYCLE_CALLBACKS = new Set(['onTestFailed', 'onTestFinished']);

function directTestLifecycleCallbackName(
  node: ESTree.Node | null | undefined,
  localCallbackNames: ReadonlySet<string>,
): string | null {
  const name = identifierName(node);
  return name !== null && localCallbackNames.has(name) ? name : null;
}

function namespaceTestLifecycleCallbackName(
  node: ESTree.Node | null | undefined,
  namespaceNames: ReadonlySet<string>,
): string | null {
  if (node?.type !== 'MemberExpression' || node.computed || node.optional) return null;

  const objectName = identifierName(node.object);
  const propertyName = nodeName(node.property);
  if (
    objectName === null ||
    propertyName === null ||
    !namespaceNames.has(objectName) ||
    !BANNED_TEST_LIFECYCLE_CALLBACKS.has(propertyName)
  ) {
    return null;
  }

  return `${objectName}.${propertyName}`;
}

function testLifecycleCallbackName(
  node: ESTree.Node | null | undefined,
  localCallbackNames: ReadonlySet<string>,
  namespaceNames: ReadonlySet<string>,
): string | null {
  return (
    directTestLifecycleCallbackName(node, localCallbackNames) ??
    namespaceTestLifecycleCallbackName(node, namespaceNames)
  );
}

/** Disallow Vitest per-test lifecycle callbacks that vitest/no-hooks does not cover. */
export const noVitestTestLifecycleCallbacksRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow Vitest per-test lifecycle callbacks that are not covered by vitest/no-hooks.',
    },
    messages: {
      noTestLifecycleCallback:
        '{{name}}() is banned in spec files. Use a test.extend context fixture and register cleanup with fixture onCleanup.',
    },
  },
  create(context) {
    const localCallbackNames = new Set<string>();
    const namespaceNames = new Set<string>();

    return {
      ImportDeclaration(node) {
        collectTestFrameworkImport(
          node,
          BANNED_TEST_LIFECYCLE_CALLBACKS,
          localCallbackNames,
          namespaceNames,
        );
      },
      CallExpression(node) {
        const name = testLifecycleCallbackName(node.callee, localCallbackNames, namespaceNames);
        if (name === null) return;

        context.report({
          node,
          messageId: 'noTestLifecycleCallback',
          data: { name },
        });
      },
    };
  },
});
