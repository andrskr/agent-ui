import { AppShell } from '@astryxdesign/core/AppShell';
import { Theme } from '@astryxdesign/core/theme';
import { neutralTheme } from '@astryxdesign/theme-neutral/built';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app.tsx';

import './styles.css';

const root = document.querySelector('#root');

if (root === null) {
  throw new Error('The application root is missing.');
}

createRoot(root).render(
  <StrictMode>
    <Theme theme={neutralTheme} mode="system">
      <AppShell contentPadding={6}>
        <App />
      </AppShell>
    </Theme>
  </StrictMode>,
);
