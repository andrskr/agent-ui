import { defineRule } from '@oxlint/plugins';
import type { Context, ESTree, Scope, Variable } from '@oxlint/plugins';

interface PreferArrowCallbackOptions {
  allowNamedFunctions: boolean;
  allowUnboundThis: boolean;
}

interface PreferArrowCallbackOptionsInput {
  readonly allowNamedFunctions?: unknown;
  readonly allowUnboundThis?: unknown;
}

interface ScopeInfo {
  this: boolean;
  super: boolean;
  meta: boolean;
}

interface CallbackInfo {
  isCallback: boolean;
  isLexicalThis: boolean;
}

type CallLikeExpression = ESTree.CallExpression | ESTree.NewExpression;

interface BindCallInfo {
  callNode: CallLikeExpression;
  isLexicalThis: boolean;
}

interface CallbackWalkState {
  bound: boolean;
  currentNode: ESTree.Node;
  parent: ESTree.Node | null;
  result: CallbackInfo;
}

const DEFAULT_OPTIONS: PreferArrowCallbackOptions = {
  allowNamedFunctions: false,
  allowUnboundThis: true,
};

function isPreferArrowCallbackOptionsInput(
  value: unknown,
): value is PreferArrowCallbackOptionsInput {
  return typeof value === 'object' && value !== null;
}

function readOptions(raw: unknown): PreferArrowCallbackOptions {
  if (!isPreferArrowCallbackOptionsInput(raw)) return DEFAULT_OPTIONS;

  return {
    allowNamedFunctions:
      typeof raw.allowNamedFunctions === 'boolean'
        ? raw.allowNamedFunctions
        : DEFAULT_OPTIONS.allowNamedFunctions,
    allowUnboundThis:
      typeof raw.allowUnboundThis === 'boolean'
        ? raw.allowUnboundThis
        : DEFAULT_OPTIONS.allowUnboundThis,
  };
}

function isFunctionName(variable: Variable | undefined): boolean {
  return variable?.defs[0]?.type === 'FunctionName';
}

function getVariableOfArguments(scope: Scope | null): Variable | null {
  for (const variable of scope?.variables ?? []) {
    if (variable.name === 'arguments') {
      return variable.identifiers.length === 0 ? variable : null;
    }
  }

  return null;
}

function asMemberExpression(node: ESTree.Node | null): ESTree.MemberExpression | null {
  return node !== null && node.type === 'MemberExpression' ? node : null;
}

function callForCallee(node: ESTree.Node): CallLikeExpression | null {
  const { parent } = node;
  if (parent === null || (parent.type !== 'CallExpression' && parent.type !== 'NewExpression')) {
    return null;
  }
  return parent.callee === node ? parent : null;
}

function isTransparentWrapper(node: ESTree.Node): boolean {
  return (
    node.type === 'LogicalExpression' ||
    node.type === 'ChainExpression' ||
    node.type === 'ConditionalExpression'
  );
}

function isBindMember(memberNode: ESTree.MemberExpression, currentNode: ESTree.Node): boolean {
  return (
    memberNode.object === currentNode &&
    memberNode.computed === false &&
    memberNode.property.name === 'bind'
  );
}

function bindCallNode(memberNode: ESTree.MemberExpression): CallLikeExpression | null {
  const maybeCallee = memberNode.parent.type === 'ChainExpression' ? memberNode.parent : memberNode;
  return callForCallee(maybeCallee);
}

function isSingleThisArgument(callNode: CallLikeExpression): boolean {
  return callNode.arguments.length === 1 && callNode.arguments[0]?.type === 'ThisExpression';
}

function bindCallInfo(
  memberNode: ESTree.MemberExpression,
  currentNode: ESTree.Node,
): BindCallInfo | null {
  if (!isBindMember(memberNode, currentNode)) return null;

  const callNode = bindCallNode(memberNode);
  if (callNode === null) return null;
  return {
    callNode,
    isLexicalThis: isSingleThisArgument(callNode),
  };
}

function isCallLike(node: ESTree.Node): node is CallLikeExpression {
  return node.type === 'CallExpression' || node.type === 'NewExpression';
}

function advanceThroughTransparentParent(state: CallbackWalkState, parent: ESTree.Node): void {
  state.currentNode = parent;
  state.parent = parent.parent;
}

function advanceThroughBindCall(state: CallbackWalkState): boolean {
  const memberNode = asMemberExpression(state.parent);
  if (memberNode === null) return false;

  const info = bindCallInfo(memberNode, state.currentNode);
  if (info === null) return false;

  if (!state.bound) {
    state.bound = true;
    state.result.isLexicalThis = info.isLexicalThis;
  }
  state.currentNode = info.callNode;
  state.parent = info.callNode.parent;
  return true;
}

function markCallbackFromCall(
  state: CallbackWalkState,
  callNode: CallLikeExpression,
): CallbackInfo {
  if (callNode.callee !== state.currentNode) state.result.isCallback = true;
  return state.result;
}

function getCallbackInfo(node: ESTree.Node): CallbackInfo {
  const state: CallbackWalkState = {
    bound: false,
    currentNode: node,
    parent: node.parent,
    result: { isCallback: false, isLexicalThis: false },
  };

  while (state.parent !== null) {
    if (isTransparentWrapper(state.parent)) {
      advanceThroughTransparentParent(state, state.parent);
      continue;
    }

    if (asMemberExpression(state.parent) !== null && advanceThroughBindCall(state)) continue;

    if (isCallLike(state.parent)) {
      return markCallbackFromCall(state, state.parent);
    }

    return state.result;
  }

  return state.result;
}

function shouldSkipFunctionExpression(
  context: Context,
  node: ESTree.Function,
  options: PreferArrowCallbackOptions,
): boolean {
  if (options.allowNamedFunctions && node.id !== null) return true;

  if (node.generator) return true;

  const nameVariable = context.sourceCode.getDeclaredVariables(node)[0];
  if (isFunctionName(nameVariable) && nameVariable.references.length > 0) return true;

  const argumentsVariable = getVariableOfArguments(context.sourceCode.getScope(node));
  return argumentsVariable !== null && argumentsVariable.references.length > 0;
}

function shouldReportCallback(
  options: PreferArrowCallbackOptions,
  scopeInfo: ScopeInfo,
  callbackInfo: CallbackInfo,
): boolean {
  if (!callbackInfo.isCallback) return false;
  if (scopeInfo.super || scopeInfo.meta) return false;
  if (options.allowUnboundThis && scopeInfo.this && !callbackInfo.isLexicalThis) return false;
  return true;
}

/** Require arrow functions for callbacks that can safely use lexical scope. */
export const preferArrowCallbackRule = defineRule({
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require arrow functions for callbacks that can safely use lexical scope.',
    },
    messages: {
      preferArrowCallback: 'Unexpected function expression.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          allowNamedFunctions: { type: 'boolean' },
          allowUnboundThis: { type: 'boolean' },
        },
        additionalProperties: false,
      },
    ],
  },
  create(context) {
    const options = readOptions(context.options[0]);
    let stack: ScopeInfo[] = [];

    const enterScope = () => {
      stack.push({ this: false, super: false, meta: false });
    };

    const exitScope = (): ScopeInfo => stack.pop() ?? { this: false, super: false, meta: false };

    return {
      Program() {
        stack = [];
      },
      ThisExpression() {
        const info = stack.at(-1);
        if (info !== undefined) info.this = true;
      },
      Super() {
        const info = stack.at(-1);
        if (info !== undefined) info.super = true;
      },
      MetaProperty(node) {
        const info = stack.at(-1);
        if (info !== undefined && node.meta.name === 'new') info.meta = true;
      },
      FunctionDeclaration: enterScope,
      'FunctionDeclaration:exit': exitScope,
      FunctionExpression: enterScope,
      'FunctionExpression:exit'(node) {
        const scopeInfo = exitScope();

        if (shouldSkipFunctionExpression(context, node, options)) return;

        const callbackInfo = getCallbackInfo(node);
        if (shouldReportCallback(options, scopeInfo, callbackInfo)) {
          context.report({
            node,
            messageId: 'preferArrowCallback',
          });
        }
      },
    };
  },
});
