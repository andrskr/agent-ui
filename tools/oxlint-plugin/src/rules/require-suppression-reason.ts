import type { Comment } from '@oxlint/plugins';
import { defineRule } from '@oxlint/plugins';

interface ParsedSuppressionDirective {
  directive: string;
  ruleList: string;
  reason: string;
}

const DEFAULT_MINIMUM_DESCRIPTION_LENGTH = 12;
const DISABLE_DIRECTIVE_RE =
  /^(?<directive>(?:eslint|oxlint)-disable(?:-next-line|-line)?)\b(?<rest>.*)$/u;

function parseSuppressionDirective(comment: Comment): ParsedSuppressionDirective | null {
  const payload = comment.value.trim();
  if (payload.length === 0) return null;

  const match = DISABLE_DIRECTIVE_RE.exec(payload);
  if (match?.groups === undefined) return null;

  const rest = match.groups.rest.trim();
  const separatorIndex = rest.indexOf('--');
  const ruleList = separatorIndex === -1 ? rest : rest.slice(0, separatorIndex).trim();
  const reason = separatorIndex === -1 ? '' : rest.slice(separatorIndex + 2).trim();

  return { directive: match.groups.directive, ruleList, reason };
}

function explicitRuleIds(ruleList: string): string[] {
  return ruleList.split(',').flatMap((ruleId) => {
    const trimmed = ruleId.trim();
    return trimmed.length > 0 ? [trimmed] : [];
  });
}

interface RequireSuppressionReasonOptions {
  minimumDescriptionLength: number;
}

function readOptions(raw: unknown): RequireSuppressionReasonOptions {
  if (typeof raw === 'object' && raw !== null && 'minimumDescriptionLength' in raw) {
    const { minimumDescriptionLength } = raw;
    if (typeof minimumDescriptionLength === 'number') return { minimumDescriptionLength };
  }
  return { minimumDescriptionLength: DEFAULT_MINIMUM_DESCRIPTION_LENGTH };
}

/** Require lint suppression directives to name explicit rules and carry a reason after `--`. */
export const requireSuppressionReasonRule = defineRule({
  meta: {
    type: 'problem',
    docs: {
      description:
        'Require lint suppression directives to name explicit rules and include a reason.',
    },
    messages: {
      requireRuleIds: '{{directive}} must name explicit rule IDs instead of disabling every rule.',
      requireReason: '{{directive}} for {{rules}} must include a reason after `--`.',
      requireLongerReason:
        '{{directive}} reason must be at least {{minimumDescriptionLength}} characters.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          minimumDescriptionLength: { type: 'number' },
        },
        additionalProperties: false,
      },
    ],
  },
  create(context) {
    const { minimumDescriptionLength: minimumLength } = readOptions(context.options[0]);

    return {
      Program() {
        for (const comment of context.sourceCode.getAllComments()) {
          const directive = parseSuppressionDirective(comment);
          if (directive === null) continue;

          const ruleIds = explicitRuleIds(directive.ruleList);
          if (ruleIds.length === 0) {
            context.report({
              node: comment,
              messageId: 'requireRuleIds',
              data: { directive: directive.directive },
            });
            continue;
          }

          if (directive.reason.length === 0) {
            context.report({
              node: comment,
              messageId: 'requireReason',
              data: { directive: directive.directive, rules: ruleIds.join(', ') },
            });
            continue;
          }

          if (directive.reason.length < minimumLength) {
            context.report({
              node: comment,
              messageId: 'requireLongerReason',
              data: { directive: directive.directive, minimumDescriptionLength: minimumLength },
            });
          }
        }
      },
    };
  },
});
