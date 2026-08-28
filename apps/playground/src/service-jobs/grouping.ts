import { customerName, technicianName } from '#/service-jobs/data.ts';
import {
  priorityLabelOrKey,
  prioritySeverity,
  statusLabelOrKey,
  statusSeverity,
} from '#/service-jobs/format.ts';
import type { GroupField, Job } from '#/service-jobs/service-jobs-types.ts';

export const ALL_GROUP_KEY = '__all__';

export function groupKeyFor(job: Job, field: GroupField): string {
  switch (field) {
    case 'none':
      return ALL_GROUP_KEY;
    case 'status':
      return job.status;
    case 'priority':
      return job.priority;
    case 'technician':
      return job.technicianId;
    case 'customer':
      return job.customerId;
    default:
      return ALL_GROUP_KEY;
  }
}

export function groupLabelFor(field: GroupField, key: string): string {
  switch (field) {
    case 'none':
      return 'All jobs';
    case 'status':
      return statusLabelOrKey(key);
    case 'priority':
      return priorityLabelOrKey(key);
    case 'technician':
      return technicianName(key);
    case 'customer':
      return customerName(key);
    default:
      return key;
  }
}

/**
 * Stable group order over the full (filtered + sorted) list: severity order for status/priority,
 * alphabetical for customer/technician. Computed once up front so the order never reshuffles as
 * more batches load.
 */
export function groupOrderFor(field: GroupField, jobs: Job[]): string[] {
  const seen: string[] = [];
  const seenSet = new Set<string>();
  for (const job of jobs) {
    const key = groupKeyFor(job, field);
    if (!seenSet.has(key)) {
      seenSet.add(key);
      seen.push(key);
    }
  }
  // `seen` was just built above and isn't used elsewhere, so sorting it in place is safe.
  // toSorted() would avoid the mutation question entirely but needs an ES2023 lib target,
  // which this project doesn't set.
  if (field === 'status') {
    // eslint-disable-next-line unicorn/no-array-sort -- see comment above; `seen` is a fresh, unshared local array.
    return seen.sort((a, b) => statusSeverity(a) - statusSeverity(b));
  }
  if (field === 'priority') {
    // eslint-disable-next-line unicorn/no-array-sort -- see comment above; `seen` is a fresh, unshared local array.
    return seen.sort((a, b) => prioritySeverity(a) - prioritySeverity(b));
  }
  if (field === 'customer' || field === 'technician') {
    // eslint-disable-next-line unicorn/no-array-sort -- see comment above; `seen` is a fresh, unshared local array.
    return seen.sort((a, b) => groupLabelFor(field, a).localeCompare(groupLabelFor(field, b)));
  }
  return seen;
}
