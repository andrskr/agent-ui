import { defineRule } from '@oxlint/plugins';

const GENERIC_BASENAMES = new Set([
  'common',
  'constants',
  'helpers',
  'misc',
  'shared',
  'types',
  'util',
  'utils',
]);

function basenameWithoutExtension(filename: string): string {
  const normalized = filename.replaceAll('\\', '/');
  const basename = normalized.slice(normalized.lastIndexOf('/') + 1);
  const firstDot = basename.indexOf('.');
  return firstDot === -1 ? basename : basename.slice(0, firstDot);
}

/** Disallow catch-all module filenames (utils, helpers, types, common, shared, misc, ...). */
export const noGenericModuleFilenamesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow generic module filenames that tend to become catch-all files.',
    },
    messages: {
      genericFilename:
        'Avoid generic module filenames like utils, helpers, types, common, shared, or misc.',
    },
  },
  create(context) {
    return {
      Program(node) {
        if (GENERIC_BASENAMES.has(basenameWithoutExtension(context.filename))) {
          context.report({ node, messageId: 'genericFilename' });
        }
      },
    };
  },
});
