import type { PowerSearchFilter } from '@astryxdesign/core/PowerSearch';

export type JobStatus = 'scheduled' | 'in_progress' | 'on_hold' | 'overdue' | 'completed';
export type JobPriority = 'urgent' | 'high' | 'normal';

export const JOB_STATUSES: readonly JobStatus[] = [
  'scheduled',
  'in_progress',
  'on_hold',
  'overdue',
  'completed',
];
export const JOB_PRIORITIES: readonly JobPriority[] = ['urgent', 'high', 'normal'];
export const DENSITIES: readonly Density[] = ['compact', 'balanced', 'spacious'];
export const GROUP_FIELDS: readonly GroupField[] = [
  'none',
  'status',
  'priority',
  'technician',
  'customer',
];

function includesGuard<TValue extends string>(values: readonly TValue[]) {
  const asStrings: readonly string[] = values;
  return (value: string): value is TValue => asStrings.includes(value);
}

export const isJobStatus = includesGuard(JOB_STATUSES);
export const isJobPriority = includesGuard(JOB_PRIORITIES);
export const isDensity = includesGuard(DENSITIES);
export const isGroupField = includesGuard(GROUP_FIELDS);

export function stickyCountFromString(value: string): 0 | 1 | 2 {
  if (value === '1') return 1;
  if (value === '2') return 2;
  return 0;
}

export interface Technician {
  id: string;
  name: string;
}

export interface Customer {
  id: string;
  name: string;
}

export interface Job extends Record<string, unknown> {
  id: string;
  jobNumber: string;
  title: string;
  customerId: string;
  technicianId: string;
  status: JobStatus;
  priority: JobPriority;
  scheduledAt: string;
  quote: number;
}

export type ColumnKey =
  | 'job'
  | 'technician'
  | 'customer'
  | 'status'
  | 'priority'
  | 'scheduled'
  | 'quote';

export type Density = 'compact' | 'balanced' | 'spacious';

export type GroupField = 'none' | 'status' | 'priority' | 'technician' | 'customer';

export interface SortState {
  columnKey: ColumnKey;
  direction: 'ascending' | 'descending';
}

export interface ViewOptions {
  columnOrder: ColumnKey[];
  activeColumns: ColumnKey[];
  density: Density;
  stickyStart: 0 | 1 | 2;
  stickyEnd: 0 | 1 | 2;
  grouping: GroupField;
}

/** The whole screen a saved view restores: filters, columns, density, sticky edges, grouping. */
export interface ServiceJobsViewState {
  filters: PowerSearchFilter[];
  sort: SortState;
  options: ViewOptions;
}

export interface SavedView {
  id: string;
  name: string;
  state: ServiceJobsViewState;
}
