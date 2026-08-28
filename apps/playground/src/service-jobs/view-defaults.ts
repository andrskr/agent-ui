import { DEFAULT_COLUMN_ORDER } from '#/service-jobs/columns.tsx';
import type { SavedView, ServiceJobsViewState } from '#/service-jobs/service-jobs-types.ts';

export const DEFAULT_VIEW_STATE: ServiceJobsViewState = {
  filters: [],
  sort: { columnKey: 'scheduled', direction: 'ascending' },
  options: {
    columnOrder: DEFAULT_COLUMN_ORDER,
    activeColumns: DEFAULT_COLUMN_ORDER,
    density: 'balanced',
    stickyStart: 1,
    stickyEnd: 0,
    grouping: 'customer',
  },
};

export const BUILT_IN_SAVED_VIEWS: SavedView[] = [
  {
    id: 'view-by-status',
    name: 'By status',
    state: {
      filters: [],
      sort: { columnKey: 'priority', direction: 'ascending' },
      options: {
        columnOrder: DEFAULT_COLUMN_ORDER,
        activeColumns: DEFAULT_COLUMN_ORDER,
        density: 'balanced',
        stickyStart: 1,
        stickyEnd: 0,
        grouping: 'status',
      },
    },
  },
  {
    id: 'view-compact-quote',
    name: 'Compact, quote pinned',
    state: {
      filters: [],
      sort: { columnKey: 'scheduled', direction: 'ascending' },
      options: {
        columnOrder: DEFAULT_COLUMN_ORDER,
        activeColumns: DEFAULT_COLUMN_ORDER,
        density: 'compact',
        stickyStart: 1,
        stickyEnd: 1,
        grouping: 'none',
      },
    },
  },
];
