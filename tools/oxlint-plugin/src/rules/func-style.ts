import { defineRule } from '@oxlint/plugins';
import type { ESTree, Fix, Fixer, SourceCode } from '@oxlint/plugins';

type FunctionNode = ESTree.ArrowFunctionExpression | ESTree.Function;

type FunctionStyle = 'declaration' | 'expression';
type NamedExportStyle = FunctionStyle | 'ignore';

interface FuncStyleOptions {
  allowArrowFunctions: boolean;
  allowTypeAnnotation: boolean;
  overrides: {
    namedExports?: NamedExportStyle;
  };
}

interface FuncStyleOptionsInput {
  readonly allowArrowFunctions?: unknown;
  readonly allowTypeAnnotation?: unknown;
  readonly namedExports?: unknown;
  readonly overrides?: unknown;
}

const DEFAULT_OPTIONS: FuncStyleOptions = {
  allowArrowFunctions: false,
  allowTypeAnnotation: false,
  overrides: {},
};

function isFuncStyleOptionsInput(value: unknown): value is FuncStyleOptionsInput {
  return typeof value === 'object' && value !== null;
}

function namedExportStyle(value: unknown): NamedExportStyle | undefined {
  return value === 'declaration' || value === 'expression' || value === 'ignore'
    ? value
    : undefined;
}

function readOptions(raw: unknown): FuncStyleOptions {
  if (!isFuncStyleOptionsInput(raw)) return DEFAULT_OPTIONS;

  return {
    allowArrowFunctions:
      typeof raw.allowArrowFunctions === 'boolean'
        ? raw.allowArrowFunctions
        : DEFAULT_OPTIONS.allowArrowFunctions,
    allowTypeAnnotation:
      typeof raw.allowTypeAnnotation === 'boolean'
        ? raw.allowTypeAnnotation
        : DEFAULT_OPTIONS.allowTypeAnnotation,
    overrides: {
      namedExports: namedExportStyle(
        isFuncStyleOptionsInput(raw.overrides) ? raw.overrides.namedExports : undefined,
      ),
    },
  };
}

// The declaration name is only usable for a fix when it is a plain identifier binding.
function declaratorName(declarator: ESTree.VariableDeclarator): string | null {
  return declarator.id.type === 'Identifier' ? declarator.id.name : null;
}

function variableDeclarationFor(node: ESTree.Node): ESTree.VariableDeclaration | null {
  const parent = node.parent;
  return parent?.type === 'VariableDeclarator' && parent.parent.type === 'VariableDeclaration'
    ? parent.parent
    : null;
}

function isNamedExportVariableDeclarator(declarator: ESTree.VariableDeclarator): boolean {
  return declarator.parent.parent?.type === 'ExportNamedDeclaration';
}

function namedExportOverride(
  declarator: ESTree.VariableDeclarator,
  options: FuncStyleOptions,
): NamedExportStyle | undefined {
  return isNamedExportVariableDeclarator(declarator) ? options.overrides.namedExports : undefined;
}

function hasAllowedTypeAnnotation(
  declarator: ESTree.VariableDeclarator,
  allowTypeAnnotation: boolean,
): boolean {
  return (
    allowTypeAnnotation &&
    declarator.id.type === 'Identifier' &&
    declarator.id.typeAnnotation !== null &&
    declarator.id.typeAnnotation !== undefined
  );
}

// A function declaration is an overload signature group when a matching TSDeclareFunction sits
// beside it, so it must not be rewritten to an expression.
function statementList(node: ESTree.Node | null | undefined): readonly ESTree.Node[] {
  return node !== null && node !== undefined && 'body' in node && Array.isArray(node.body)
    ? node.body
    : [];
}

function declaresFunction(member: ESTree.Node, functionName: string): boolean {
  return (
    member.type === 'TSDeclareFunction' && member.id !== null && member.id.name === functionName
  );
}

function hasBodyOverload(body: readonly ESTree.Node[], functionName: string): boolean {
  return body.some((member) => declaresFunction(member, functionName));
}

function hasExportedOverload(body: readonly ESTree.Node[], functionName: string): boolean {
  return body.some(
    (member) =>
      member.type === 'ExportNamedDeclaration' &&
      member.declaration !== null &&
      member.declaration !== undefined &&
      declaresFunction(member.declaration, functionName),
  );
}

function hasSwitchCaseOverload(node: ESTree.Node, functionName: string): boolean {
  return (
    node.type === 'SwitchStatement' &&
    node.cases.some((switchCase) => hasBodyOverload(switchCase.consequent, functionName))
  );
}

function isOverloadedFunction(node: ESTree.Function): boolean {
  const functionName = node.id?.name;
  if (functionName === undefined) return false;

  const parent = node.parent;
  if (parent.type === 'ExportNamedDeclaration') {
    return hasExportedOverload(statementList(parent.parent), functionName);
  }
  if (parent.type === 'SwitchCase') {
    return hasSwitchCaseOverload(parent.parent, functionName);
  }
  return hasBodyOverload(statementList(parent), functionName);
}

function functionText(sourceCode: SourceCode, node: FunctionNode): string {
  return sourceCode.getText(node);
}

function normalizeArrowHead(head: string): string {
  return head.trim().startsWith('async ')
    ? head.trim().slice('async '.length).trimStart()
    : head.trim();
}

function statementBodyForExpression(expression: string): string {
  return `{\n  return ${expression};\n}`;
}

// Scan a signature for the top-level `=>`, ignoring arrows nested inside brackets or strings.
function findTopLevelArrow(text: string): number {
  const depths = { '<': 0, '(': 0, '{': 0, '[': 0 };
  const openFor: Record<string, keyof typeof depths> = { '>': '<', ')': '(', '}': '{', ']': '[' };
  let quote: string | null = null;

  for (let index = 0; index < text.length - 1; index += 1) {
    const char = text[index];

    if (quote !== null) {
      if (char === '\\') index += 1;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === "'" || char === '"' || char === '`') {
      quote = char;
      continue;
    }

    if (char === '<' || char === '(' || char === '{' || char === '[') depths[char] += 1;
    else if (char in openFor && depths[openFor[char]] > 0) depths[openFor[char]] -= 1;

    const atTopLevel =
      depths['<'] === 0 && depths['('] === 0 && depths['{'] === 0 && depths['['] === 0;
    if (char === '=' && text[index + 1] === '>' && atTopLevel) return index;
  }

  return -1;
}

interface FunctionSignatureParts {
  asyncPrefix: string;
  generator: string;
  functionTail: string;
}

function parseFunctionSignature(signature: string): FunctionSignatureParts | null {
  if (/\/[/*]/.test(signature)) return null;

  const match = /^(async\s+)?function(\*)?\s*(?:[$A-Z_a-z][$\w]*\s*)?([\s\S]+)$/.exec(
    signature.trim(),
  );
  const functionTail = match?.at(3);
  if (functionTail === undefined) return null;

  return {
    asyncPrefix: match?.at(1) ?? '',
    generator: match?.at(2) ?? '',
    functionTail,
  };
}

function declarationFromArrow(
  sourceCode: SourceCode,
  name: string,
  node: ESTree.ArrowFunctionExpression,
): string | null {
  const source = functionText(sourceCode, node);
  const arrowIndex = findTopLevelArrow(source);
  if (arrowIndex === -1) return null;

  const bodyText = sourceCode.getText(node.body);
  const head = normalizeArrowHead(source.slice(0, arrowIndex));
  const prefix = `${node.async ? 'async ' : ''}function ${name}`;
  const signature = head.startsWith('<') || head.startsWith('(') ? head : `(${head})`;
  const declarationBody =
    node.body.type === 'BlockStatement' ? bodyText : statementBodyForExpression(bodyText);

  return `${prefix}${signature} ${declarationBody}`;
}

function declarationFromFunctionExpression(
  sourceCode: SourceCode,
  name: string,
  node: ESTree.Function,
): string | null {
  const source = functionText(sourceCode, node);
  const bodyText = sourceCode.getText(node.body);
  const signature = parseFunctionSignature(
    source.slice(0, source.length - bodyText.length).trimEnd(),
  );
  if (signature === null) return null;

  return `${signature.asyncPrefix}function${signature.generator} ${name}${signature.functionTail} ${bodyText}`;
}

function declarationReplacementTarget(declaration: ESTree.VariableDeclaration): {
  prefix: string;
  range: [number, number];
} {
  const exportNode =
    declaration.parent?.type === 'ExportNamedDeclaration' ? declaration.parent : null;
  return exportNode === null
    ? { prefix: '', range: declaration.range }
    : { prefix: 'export ', range: exportNode.range };
}

function declarationReplacement(
  sourceCode: SourceCode,
  declarator: ESTree.VariableDeclarator,
  functionNode: FunctionNode,
): ((fixer: Fixer) => Fix) | undefined {
  const declaration = variableDeclarationFor(functionNode);
  const name = declaratorName(declarator);
  if (declaration === null || declaration.declarations.length !== 1 || name === null) {
    return undefined;
  }

  const replacement =
    functionNode.type === 'ArrowFunctionExpression'
      ? declarationFromArrow(sourceCode, name, functionNode)
      : functionNode.type === 'FunctionExpression'
        ? declarationFromFunctionExpression(sourceCode, name, functionNode)
        : null;
  if (replacement === null) return undefined;

  const target = declarationReplacementTarget(declaration);
  return (fixer) => fixer.replaceTextRange(target.range, `${target.prefix}${replacement}`);
}

function requiresFunctionDeclaration(
  declarator: ESTree.VariableDeclarator,
  enforceDeclarations: boolean,
  options: FuncStyleOptions,
): boolean {
  if (hasAllowedTypeAnnotation(declarator, options.allowTypeAnnotation)) return false;

  const exportStyle = namedExportOverride(declarator, options);
  if (exportStyle === 'ignore' || exportStyle === 'expression') return false;
  return enforceDeclarations || exportStyle === 'declaration';
}

/** Enforce a consistent choice between function declarations and variable-assigned helpers. */
export const funcStyleRule = defineRule({
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Enforce consistent use of function declarations over variable-assigned function helpers.',
    },
    messages: {
      expression: 'Expected a function expression.',
      declaration: 'Expected a function declaration.',
    },
    schema: [
      {
        enum: ['declaration', 'expression'],
      },
      {
        type: 'object',
        properties: {
          allowArrowFunctions: { type: 'boolean' },
          allowTypeAnnotation: { type: 'boolean' },
          overrides: {
            type: 'object',
            properties: {
              namedExports: {
                enum: ['declaration', 'expression', 'ignore'],
              },
            },
            additionalProperties: false,
          },
        },
        additionalProperties: false,
      },
    ],
    fixable: 'code',
  },
  create(context) {
    const style: FunctionStyle =
      context.options[0] === 'declaration' ? 'declaration' : 'expression';
    const options = readOptions(context.options[1]);
    const enforceDeclarations = style === 'declaration';
    const stack: boolean[] = [];

    const reportDeclarationIfNeeded = (node: ESTree.Function | ESTree.ArrowFunctionExpression) => {
      const declarator = node.parent.type === 'VariableDeclarator' ? node.parent : null;
      if (
        declarator === null ||
        !requiresFunctionDeclaration(declarator, enforceDeclarations, options)
      ) {
        return;
      }

      context.report({
        node: declarator,
        messageId: 'declaration',
        fix: declarationReplacement(context.sourceCode, declarator, node),
      });
    };

    const shouldReportFunctionDeclaration = (node: ESTree.Function): boolean => {
      if (enforceDeclarations) return false;
      if (node.parent.type === 'ExportDefaultDeclaration') return false;
      if (
        options.overrides.namedExports !== undefined &&
        node.parent.type === 'ExportNamedDeclaration'
      ) {
        return false;
      }
      return !isOverloadedFunction(node);
    };

    const shouldReportNamedExportDeclaration = (node: ESTree.Function): boolean =>
      node.parent.type === 'ExportNamedDeclaration' &&
      options.overrides.namedExports === 'expression' &&
      !isOverloadedFunction(node);

    const visitor: Record<string, (node: ESTree.Node) => void> = {
      FunctionDeclaration(node) {
        if (node.type !== 'FunctionDeclaration') return;
        stack.push(false);

        if (shouldReportFunctionDeclaration(node)) {
          context.report({ node, messageId: 'expression' });
        }
        if (shouldReportNamedExportDeclaration(node)) {
          context.report({ node, messageId: 'expression' });
        }
      },
      'FunctionDeclaration:exit'() {
        stack.pop();
      },
      FunctionExpression(node) {
        if (node.type !== 'FunctionExpression') return;
        stack.push(false);
        reportDeclarationIfNeeded(node);
      },
      'FunctionExpression:exit'() {
        stack.pop();
      },
      'ThisExpression, Super, MetaProperty'(node) {
        if (stack.length === 0) return;
        if (node.type === 'MetaProperty' && node.meta.name !== 'new') return;
        stack[stack.length - 1] = true;
      },
    };

    if (!options.allowArrowFunctions) {
      visitor.ArrowFunctionExpression = () => {
        stack.push(false);
      };
      visitor['ArrowFunctionExpression:exit'] = (node) => {
        const hasLexicalReference = stack.pop() ?? false;
        if (hasLexicalReference || node.type !== 'ArrowFunctionExpression') return;
        reportDeclarationIfNeeded(node);
      };
    }

    return visitor;
  },
});
