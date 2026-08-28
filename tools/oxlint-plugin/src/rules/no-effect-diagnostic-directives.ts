import { defineRule } from '@oxlint/plugins';

const EFFECT_DIAGNOSTIC_DIRECTIVE = /@effect-diagnostics(?:-next-line)?\b/u;

/** Reject Effect diagnostic directives; Oxlint cannot report when they go unused. */
export const noEffectDiagnosticDirectivesRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Reject Effect diagnostic directives because Oxlint cannot report when they are unused.',
    },
    messages: {
      forbidden:
        'Do not suppress Effect diagnostics with directives. Fix the diagnostic or change the repository rule policy explicitly.',
    },
    schema: [],
  },
  create(context) {
    return {
      Program() {
        for (const comment of context.sourceCode.getAllComments()) {
          if (EFFECT_DIAGNOSTIC_DIRECTIVE.test(comment.value)) {
            context.report({ node: comment, messageId: 'forbidden' });
          }
        }
      },
    };
  },
});
