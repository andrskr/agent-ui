import * as stylex from '@stylexjs/stylex';
import { HeadContent, Scripts } from '@tanstack/react-router';
import type { ReactNode } from 'react';

import { themeName } from '#/theme/index.ts';

const styles = stylex.create({
  root: {
    isolation: 'isolate',
  },
});

export function Document({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html data-astryx-theme={themeName} lang="en">
      <head>
        <HeadContent />
      </head>
      <body>
        {/* oxlint-disable-next-line oxlint-plugin/no-raw-layout-elements -- This is the document
        shell, not app layout. The element exists only to open a stacking context around the whole
        tree; an Astryx layout component would impose flex/grid semantics on
        the app root. Astryx composition starts inside this element. */}
        <div {...stylex.props(styles.root)}>{children}</div>
        <Scripts />
      </body>
    </html>
  );
}
