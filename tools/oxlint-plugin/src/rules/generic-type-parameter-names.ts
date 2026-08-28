import { defineRule } from '@oxlint/plugins';

const COMMON_ONE_LETTER_REPLACEMENTS = new Map([
  ['A', 'TArgs'],
  ['C', 'TContext'],
  ['D', 'TData'],
  ['E', 'TError'],
  ['K', 'TKey'],
  ['N', 'TNode'],
  ['P', 'TProps'],
  ['R', 'TResult'],
  ['S', 'TState'],
  ['T', 'TValue'],
  ['U', 'TOther'],
  ['V', 'TValue'],
]);

function isOneLetterGeneric(name: string): boolean {
  return /^[A-Z]$/.test(name);
}

function hasRequiredPrefix(name: string): boolean {
  return /^T[A-Z0-9]/.test(name);
}

function capitalize(value: string): string {
  return value.length > 0 ? `${value[0]?.toUpperCase()}${value.slice(1)}` : value;
}

function suggestedName(name: string): string {
  const common = COMMON_ONE_LETTER_REPLACEMENTS.get(name);
  if (common !== undefined) return common;
  if (isOneLetterGeneric(name)) return `T${name}`;
  if (/^T[a-z]/.test(name)) return `T${capitalize(name.slice(1))}`;
  return hasRequiredPrefix(name) ? name : `T${capitalize(name)}`;
}

/** Require descriptive T-prefixed generic type parameter names instead of one-letter names. */
export const genericTypeParameterNamesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Require generic type parameters to use descriptive T-prefixed names instead of one-letter names.',
    },
    messages: {
      genericName: 'Generic type parameter "{{name}}" should be renamed to "{{replacement}}".',
    },
  },
  create(context) {
    return {
      TSTypeParameter(node) {
        const name = node.name.name;
        if (!isOneLetterGeneric(name) && hasRequiredPrefix(name)) return;

        context.report({
          node: node.name,
          messageId: 'genericName',
          data: { name, replacement: suggestedName(name) },
        });
      },
    };
  },
});
