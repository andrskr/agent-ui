import { createFileRoute } from '@tanstack/react-router';

import SettingsPage from '#/settings-page.tsx';

export const Route = createFileRoute('/settings')({
  component: SettingsPage,
});
