import { tester } from '#src/testing.ts';

import { preferReact19PrimitivesRule } from './prefer-react-19-primitives.ts';

tester.run('prefer-react-19-primitives', preferReact19PrimitivesRule, {
  valid: [
    {
      code: `
import { use } from 'react';

const ThemeContext = createContext(null);

function ThemeProvider({ children, value }) {
  return <ThemeContext value={value}>{children}</ThemeContext>;
}

function useTheme() {
  return use(ThemeContext);
}
`,
      filename: 'src/theme.tsx',
    },
    {
      code: `
function MyInput({ ref, ...props }) {
  return <input ref={ref} {...props} />;
}
`,
      filename: 'src/input.tsx',
    },
    {
      code: `
import { forwardRef, useContext } from 'react';

const MyInput = forwardRef(function MyInput(props, ref) {
  return <input ref={ref} {...props} />;
});

function useTheme() {
  return useContext(ThemeContext);
}
`,
      filename: 'src/configurable.tsx',
      options: [{ enforce: ['contextProvider'] }],
    },
    {
      code: `
function ThemeProvider({ children, value }) {
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
`,
      filename: 'src/theme.tsx',
      options: [{ enforce: ['forwardRef', 'useContext'] }],
    },
  ],
  invalid: [
    {
      code: `
import { forwardRef } from 'react';

const MyInput = forwardRef(function MyInput(props, ref) {
  return <input ref={ref} {...props} />;
});
`,
      filename: 'src/input.tsx',
      errors: [{ messageId: 'noForwardRef' }, { messageId: 'noForwardRef' }],
    },
    {
      code: `
import React from 'react';

const MyInput = React.forwardRef(function MyInput(props, ref) {
  return <input ref={ref} {...props} />;
});
`,
      filename: 'src/input.tsx',
      errors: [{ messageId: 'noForwardRef' }],
    },
    {
      code: `
import { useContext as readContext } from 'react';

function useTheme() {
  return readContext(ThemeContext);
}
`,
      filename: 'src/theme.tsx',
      errors: [{ messageId: 'noUseContext' }, { messageId: 'noUseContext' }],
    },
    {
      code: `
import React from 'react';

function useTheme() {
  return React.useContext(ThemeContext);
}
`,
      filename: 'src/theme.tsx',
      errors: [{ messageId: 'noUseContext' }],
    },
    {
      code: `
import * as R from 'react';

function useTheme() {
  return R.useContext(ThemeContext);
}
`,
      filename: 'src/theme.tsx',
      errors: [{ messageId: 'noUseContext' }],
    },
    {
      code: `
function ThemeProvider({ children, value }) {
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
`,
      filename: 'src/theme.tsx',
      errors: [{ messageId: 'noContextProvider' }],
    },
  ],
});
