import { defineRule } from '@oxlint/plugins';
import type { ESTree } from '@oxlint/plugins';

import { nodeName } from '#src/support/ast.ts';

type BannedReactPrimitive = 'forwardRef' | 'useContext';
type React19PrimitivePolicy = BannedReactPrimitive | 'contextProvider';

interface RuleOptions {
  enforce: React19PrimitivePolicy[];
}

interface RuleOptionsInput {
  readonly enforce?: unknown;
}

const BANNED_REACT_IMPORTS = new Set<BannedReactPrimitive>(['forwardRef', 'useContext']);
const DEFAULT_OPTIONS: RuleOptions = {
  enforce: ['forwardRef', 'useContext', 'contextProvider'],
};

function isRuleOptionsInput(value: unknown): value is RuleOptionsInput {
  return typeof value === 'object' && value !== null;
}

function isReact19PrimitivePolicy(value: unknown): value is React19PrimitivePolicy {
  return value === 'forwardRef' || value === 'useContext' || value === 'contextProvider';
}

function readOptions(raw: unknown): RuleOptions {
  if (!isRuleOptionsInput(raw) || !Array.isArray(raw.enforce)) {
    return DEFAULT_OPTIONS;
  }

  return {
    enforce: raw.enforce.filter(isReact19PrimitivePolicy),
  };
}

function bannedReactPrimitive(value: string | null): BannedReactPrimitive | null {
  if (value === 'forwardRef' || value === 'useContext') return value;
  return null;
}

function unwrapExpression(node: ESTree.Node | null | undefined): ESTree.Node | null {
  if (node === null || node === undefined) return null;

  if (node.type === 'ChainExpression' || node.type === 'TSInstantiationExpression') {
    return unwrapExpression(node.expression);
  }

  return node;
}

function staticMember(node: ESTree.Node | null | undefined): ESTree.MemberExpression | null {
  const expression = unwrapExpression(node);
  return expression?.type === 'MemberExpression' && !expression.computed ? expression : null;
}

function reactMemberName(node: ESTree.Node | null | undefined): string | null {
  const member = staticMember(node);
  return member === null ? null : nodeName(member.property);
}

function reactMemberObjectName(node: ESTree.Node | null | undefined): string | null {
  const member = staticMember(node);
  return member === null ? null : nodeName(unwrapExpression(member.object));
}

function jsxMemberName(
  node: ESTree.JSXOpeningElement['name'],
): { objectName: string; propertyName: string } | null {
  if (node.type !== 'JSXMemberExpression') return null;

  const objectName = nodeName(node.object);
  const propertyName = nodeName(node.property);
  if (objectName === null || propertyName === null) return null;

  return { objectName, propertyName };
}

function rememberReactNamespace(
  specifier: ESTree.ImportDeclaration['specifiers'][number],
  reactNamespaceLocals: Set<string>,
): boolean {
  if (
    specifier.type !== 'ImportDefaultSpecifier' &&
    specifier.type !== 'ImportNamespaceSpecifier'
  ) {
    return false;
  }

  reactNamespaceLocals.add(specifier.local.name);
  return true;
}

function enforcedBannedImport(
  specifier: ESTree.ImportDeclaration['specifiers'][number],
  enforcedPrimitives: ReadonlySet<React19PrimitivePolicy>,
): BannedReactPrimitive | null {
  if (specifier.type !== 'ImportSpecifier') return null;

  const primitive = bannedReactPrimitive(nodeName(specifier.imported));
  if (primitive === null || !BANNED_REACT_IMPORTS.has(primitive)) return null;
  return enforcedPrimitives.has(primitive) ? primitive : null;
}

function importedPrimitive(
  callee: ESTree.Node | null,
  bannedReactLocals: ReadonlyMap<string, BannedReactPrimitive>,
): BannedReactPrimitive | null {
  if (callee?.type !== 'Identifier') return null;
  return bannedReactLocals.get(callee.name) ?? null;
}

function namespacePrimitive(
  callee: ESTree.Node | null,
  reactNamespaceLocals: ReadonlySet<string>,
): string | null {
  const memberObjectName = reactMemberObjectName(callee);
  if (memberObjectName === null || !reactNamespaceLocals.has(memberObjectName)) return null;
  return reactMemberName(callee);
}

function bannedCallPrimitive(
  callee: ESTree.Node | null,
  bannedReactLocals: ReadonlyMap<string, BannedReactPrimitive>,
  reactNamespaceLocals: ReadonlySet<string>,
): BannedReactPrimitive | null {
  return bannedReactPrimitive(
    importedPrimitive(callee, bannedReactLocals) ??
      namespacePrimitive(callee, reactNamespaceLocals),
  );
}

/**
 * Enforce React 19 primitives (ref-as-a-prop, use, context value) over
 * forwardRef/useContext/Provider.
 */
export const preferReact19PrimitivesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforce React 19 primitives for refs and context instead of legacy forwardRef/useContext/provider syntax.',
    },
    messages: {
      noForwardRef:
        'Use React 19 ref-as-a-prop instead of forwardRef. Accept ref in props and pass it through directly.',
      noUseContext: 'Use the React 19 use(...) API instead of useContext(...).',
      noContextProvider:
        'Use React 19 context provider syntax: <{{contextName}} value={...}> instead of <{{contextName}}.Provider>.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          enforce: {
            type: 'array',
            items: { enum: ['forwardRef', 'useContext', 'contextProvider'] },
          },
        },
        additionalProperties: false,
      },
    ],
  },
  create(context) {
    const options = readOptions(context.options[0]);
    const enforcedPrimitives = new Set(options.enforce);
    const bannedReactLocals = new Map<string, BannedReactPrimitive>();
    const reactNamespaceLocals = new Set<string>();

    return {
      ImportDeclaration(node) {
        if (node.source.value !== 'react') return;

        for (const specifier of node.specifiers) {
          if (rememberReactNamespace(specifier, reactNamespaceLocals)) continue;

          const primitive = enforcedBannedImport(specifier, enforcedPrimitives);
          if (primitive === null) continue;

          bannedReactLocals.set(specifier.local.name, primitive);
          context.report({
            node: specifier,
            messageId: primitive === 'forwardRef' ? 'noForwardRef' : 'noUseContext',
          });
        }
      },
      CallExpression(node) {
        const callee = unwrapExpression(node.callee);
        const primitive = bannedCallPrimitive(callee, bannedReactLocals, reactNamespaceLocals);
        if (primitive === 'forwardRef' && enforcedPrimitives.has('forwardRef')) {
          context.report({ node, messageId: 'noForwardRef' });
        } else if (primitive === 'useContext' && enforcedPrimitives.has('useContext')) {
          context.report({ node, messageId: 'noUseContext' });
        }
      },
      JSXOpeningElement(node) {
        if (!enforcedPrimitives.has('contextProvider')) return;

        const name = jsxMemberName(node.name);
        if (name?.propertyName !== 'Provider') return;

        context.report({
          node,
          messageId: 'noContextProvider',
          data: { contextName: name.objectName },
        });
      },
    };
  },
});
