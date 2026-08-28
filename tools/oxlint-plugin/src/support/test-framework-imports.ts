import type { ESTree } from '@oxlint/plugins';

import { importedName } from './ast.ts';

const TEST_FRAMEWORK_PACKAGES = new Set([
  '@effect/vitest',
  '@jest/globals',
  'jest',
  'vite-plus/test',
  'vitest',
]);

/**
 * Collect the local bindings a file introduces for a set of test-framework APIs. Named imports
 * whose imported name is in `importedApiNames` add their local name to `localApiNames`; namespace
 * imports add their local name to `namespaceNames`.
 */
export function collectTestFrameworkImport(
  node: ESTree.ImportDeclaration,
  importedApiNames: ReadonlySet<string>,
  localApiNames: Set<string>,
  namespaceNames: Set<string>,
): void {
  if (!TEST_FRAMEWORK_PACKAGES.has(node.source.value)) return;

  for (const specifier of node.specifiers) {
    if (specifier.type === 'ImportSpecifier' && importedApiNames.has(importedName(specifier))) {
      localApiNames.add(specifier.local.name);
    } else if (specifier.type === 'ImportNamespaceSpecifier') {
      namespaceNames.add(specifier.local.name);
    }
  }
}
