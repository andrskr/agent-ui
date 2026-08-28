import { createFileRoute } from '@tanstack/react-router';

import { ExampleCatalogPage } from '#/examples/example-catalog-page.tsx';

export const Route = createFileRoute('/')({
  component: ExampleCatalogPage,
});
