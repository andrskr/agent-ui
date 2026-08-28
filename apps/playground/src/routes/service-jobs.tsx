import { createFileRoute } from '@tanstack/react-router';

import ServiceJobsPage from '#/service-jobs/page.tsx';

export const Route = createFileRoute('/service-jobs')({
  component: ServiceJobsPage,
});
