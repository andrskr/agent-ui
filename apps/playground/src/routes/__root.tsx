import { AppShell } from '@astryxdesign/core/AppShell';
import { createRootRoute } from '@tanstack/react-router';
import type { ReactNode } from 'react';

import { Document } from '#/app/document.tsx';
import { NotFound } from '#/app/not-found.tsx';
import { AppProviders } from '#/app/providers.tsx';

import appCss from '#/styles.css?url';
import themeCss from '#/theme/index.css?url';

const stylexDevelopmentRuntime = '/@id/virtual:stylex:runtime';
const stylexDevelopmentStylesheet = '/virtual:stylex.css';

export const Route = createRootRoute({
  head: () => ({
    meta: [
      // eslint-disable-next-line unicorn/text-encoding-identifier-case -- HTML charset attribute, not a Node.js encoding identifier
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { title: 'Something Something UI' },
    ],
    links: [
      { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
      { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossOrigin: 'anonymous' },
      {
        rel: 'stylesheet',
        href: 'https://fonts.googleapis.com/css2?family=Figtree:wght@300..900&display=swap',
      },
      { rel: 'stylesheet', href: appCss },
      { rel: 'stylesheet', href: themeCss },
      ...(import.meta.env.DEV ? [{ rel: 'stylesheet', href: stylexDevelopmentStylesheet }] : []),
    ],
    scripts: import.meta.env.DEV
      ? [{ defer: true, src: stylexDevelopmentRuntime, type: 'module' as const }]
      : [],
  }),
  notFoundComponent: NotFound,
  shellComponent: RootShell,
});

function RootShell({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <Document>
      <AppProviders>
        <AppShell contentPadding={4} height="fill">
          {children}
        </AppShell>
      </AppProviders>
    </Document>
  );
}
