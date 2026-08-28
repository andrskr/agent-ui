import type { ESTree, Scope, SourceCode, Variable } from '@oxlint/plugins';

/** Read a string Literal's value, or null when the node is not a string literal. */
function literalString(node: ESTree.Node | null | undefined): string | null {
  return node?.type === 'Literal' && typeof node.value === 'string' ? node.value : null;
}

/**
 * Read the plain name a node contributes: an identifier's name, a private/JSX identifier's name, or
 * a string literal's value. Returns null for everything else.
 */
export function nodeName(node: ESTree.Node | null | undefined): string | null {
  if (node === null || node === undefined) return null;
  if (node.type === 'Identifier' || node.type === 'JSXIdentifier') return node.name;
  if (node.type === 'PrivateIdentifier') return node.name;
  return literalString(node);
}

/** Read a node's name only when it is a plain identifier, otherwise null. */
export function identifierName(node: ESTree.Node | null | undefined): string | null {
  return node?.type === 'Identifier' ? node.name : null;
}

/** Read the imported name of an import specifier, whether it is an identifier or a string literal. */
export function importedName(specifier: ESTree.ImportSpecifier): string {
  return specifier.imported.type === 'Identifier'
    ? specifier.imported.name
    : specifier.imported.value;
}

/** Resolve an identifier reference to its declared variable by walking the scope chain. */
export function resolveVariable(
  sourceCode: SourceCode,
  identifier: ESTree.IdentifierReference,
): Variable | null {
  let scope: Scope | null = sourceCode.getScope(identifier);
  while (scope !== null) {
    const variable = scope.set.get(identifier.name);
    if (variable !== undefined) return variable;
    scope = scope.upper;
  }
  return null;
}
