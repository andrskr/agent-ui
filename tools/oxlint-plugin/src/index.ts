import { definePlugin } from '@oxlint/plugins';

import { funcStyleRule } from './rules/func-style.ts';
import { genericTypeParameterNamesRule } from './rules/generic-type-parameter-names.ts';
import { noChainedTypeAssertionsRule } from './rules/no-chained-type-assertions.ts';
import { noConditionalEmptyObjectSpreadRule } from './rules/no-conditional-empty-object-spread.ts';
import { noEffectDiagnosticDirectivesRule } from './rules/no-effect-diagnostic-directives.ts';
import { noGenericModuleFilenamesRule } from './rules/no-generic-module-filenames.ts';
import { noIsRecordUtilityRule } from './rules/no-is-record-utility.ts';
import { noKnownValueWideningRule } from './rules/no-known-value-widening.ts';
import { noLocalViewportMediaQueriesRule } from './rules/no-local-viewport-media-queries.ts';
import { noModuleMockingRule } from './rules/no-module-mocking.ts';
import { noObjectParametersRule } from './rules/no-object-parameters.ts';
import { noRawLayoutElementsRule } from './rules/no-raw-layout-elements.ts';
import { noReflectApplyRule } from './rules/no-reflect-apply.ts';
import { noReflectGetRule } from './rules/no-reflect-get.ts';
import { noRuntimeTypeofRule } from './rules/no-runtime-typeof.ts';
import { noForbiddenTermInSymbolNamesRule } from './rules/no-shape-in-symbol-names.ts';
import { noStyleEscapeHatchesRule } from './rules/no-style-escape-hatches.ts';
import { noTestContainersRule } from './rules/no-test-containers.ts';
import { noUnknownParametersRule } from './rules/no-unknown-parameters.ts';
import { noUnknownReturnsRule } from './rules/no-unknown-returns.ts';
import { noUnknownTypeAliasesRule } from './rules/no-unknown-type-aliases.ts';
import { noUnsafeDictionaryTypeRule } from './rules/no-unsafe-dictionary-type.ts';
import { noVitestTestLifecycleCallbacksRule } from './rules/no-vitest-test-lifecycle-callbacks.ts';
import { noWidenThenAssertRule } from './rules/no-widen-then-assert.ts';
import { preferArrowCallbackRule } from './rules/prefer-arrow-callback.ts';
import { preferReact19PrimitivesRule } from './rules/prefer-react-19-primitives.ts';
import { requireSafetyCommentForTypeAssertionRule } from './rules/require-safety-comment-for-type-assertion.ts';
import { requireSuppressionReasonRule } from './rules/require-suppression-reason.ts';

export default definePlugin({
  meta: {
    name: 'oxlint-plugin',
  },
  rules: {
    'func-style': funcStyleRule,
    'generic-type-parameter-names': genericTypeParameterNamesRule,
    'no-chained-type-assertions': noChainedTypeAssertionsRule,
    'no-conditional-empty-object-spread': noConditionalEmptyObjectSpreadRule,
    'no-effect-diagnostic-directives': noEffectDiagnosticDirectivesRule,
    'no-generic-module-filenames': noGenericModuleFilenamesRule,
    'no-is-record-utility': noIsRecordUtilityRule,
    'no-known-value-widening': noKnownValueWideningRule,
    'no-local-viewport-media-queries': noLocalViewportMediaQueriesRule,
    'no-module-mocking': noModuleMockingRule,
    'no-object-parameters': noObjectParametersRule,
    'no-raw-layout-elements': noRawLayoutElementsRule,
    'no-reflect-apply': noReflectApplyRule,
    'no-reflect-get': noReflectGetRule,
    'no-runtime-typeof': noRuntimeTypeofRule,
    'no-shape-in-symbol-names': noForbiddenTermInSymbolNamesRule,
    'no-style-escape-hatches': noStyleEscapeHatchesRule,
    'no-test-containers': noTestContainersRule,
    'no-unknown-parameters': noUnknownParametersRule,
    'no-unknown-returns': noUnknownReturnsRule,
    'no-unknown-type-aliases': noUnknownTypeAliasesRule,
    'no-unsafe-dictionary-type': noUnsafeDictionaryTypeRule,
    'no-vitest-test-lifecycle-callbacks': noVitestTestLifecycleCallbacksRule,
    'no-widen-then-assert': noWidenThenAssertRule,
    'prefer-arrow-callback': preferArrowCallbackRule,
    'prefer-react-19-primitives': preferReact19PrimitivesRule,
    'require-safety-comment-for-type-assertion': requireSafetyCommentForTypeAssertionRule,
    'require-suppression-reason': requireSuppressionReasonRule,
  },
});
