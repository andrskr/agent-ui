import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

import { identifierName, nodeName } from '#src/support/ast.ts';
import { collectTestFrameworkImport } from '#src/support/test-framework-imports.ts';

const BANNED_CONTAINER_EXPORTS = new Set(['context', 'describe', 'suite']);

function isDirectNamespaceContainer(
  objectName: string | null,
  propertyName: string | null,
  namespaceNames: ReadonlySet<string>,
): boolean {
  return (
    objectName !== null &&
    propertyName !== null &&
    namespaceNames.has(objectName) &&
    BANNED_CONTAINER_EXPORTS.has(propertyName)
  );
}

function isContainerModifier(propertyName: string | null): boolean {
  return propertyName === 'only' || propertyName === 'skip';
}

function containerName(
  node: ESTree.Node | null | undefined,
  localContainerNames: ReadonlySet<string>,
  namespaceNames: ReadonlySet<string>,
): string | null {
  const directName = identifierName(node);
  if (directName !== null && localContainerNames.has(directName)) return directName;

  if (node?.type !== 'MemberExpression' || node.computed || node.optional) return null;

  const propertyName = nodeName(node.property);
  const objectName = identifierName(node.object);
  if (isDirectNamespaceContainer(objectName, propertyName, namespaceNames)) {
    return `${objectName}.${propertyName}`;
  }

  const nestedName = containerName(node.object, localContainerNames, namespaceNames);
  if (nestedName !== null && isContainerModifier(propertyName)) {
    return `${nestedName}.${propertyName}`;
  }

  return null;
}

/** Disallow test container functions so spec files stay flat. */
export const noTestContainersRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow test container functions so spec files stay flat.',
    },
    messages: {
      noContainer:
        '{{name}}() is banned in spec files. Keep tests flat and move repeated setup into helpers, fixtures, or test.extend context.',
    },
  },
  create(context) {
    const localContainerNames = new Set<string>();
    const namespaceNames = new Set<string>();

    return {
      ImportDeclaration(node) {
        collectTestFrameworkImport(
          node,
          BANNED_CONTAINER_EXPORTS,
          localContainerNames,
          namespaceNames,
        );
      },
      CallExpression(node) {
        const name = containerName(node.callee, localContainerNames, namespaceNames);
        if (name === null) return;

        context.report({
          node,
          messageId: 'noContainer',
          data: { name },
        });
      },
    };
  },
});
