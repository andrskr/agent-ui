import { defineTheme } from '@astryxdesign/core/theme';
import { neutralTheme } from '@astryxdesign/theme-neutral';

import { icons } from './icons.ts';

export const theme = defineTheme({
  name: 'app',
  extends: neutralTheme,
  icons: icons,
});
