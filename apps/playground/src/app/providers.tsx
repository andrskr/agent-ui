import { LinkProvider } from '@astryxdesign/core/Link';
import { Theme } from '@astryxdesign/core/theme';
import type { ReactNode } from 'react';

import { theme } from '#/theme/index.ts';

import { RouterLink } from './router-link.tsx';

export function AppProviders({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <LinkProvider component={RouterLink}>
      <Theme mode="system" theme={theme}>
        {children}
      </Theme>
    </LinkProvider>
  );
}
