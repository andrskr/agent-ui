import type { JobPriority, JobStatus } from '#/service-jobs/service-jobs-types.ts';
import { isJobPriority, isJobStatus } from '#/service-jobs/service-jobs-types.ts';

export const STATUS_LABEL = {
  overdue: 'Overdue',
  in_progress: 'In progress',
  on_hold: 'On hold',
  scheduled: 'Scheduled',
  completed: 'Completed',
} satisfies Record<JobStatus, string>;

// Severity order for triage: needs-attention states sort first, completed last.
export const STATUS_SEVERITY = {
  overdue: 0,
  in_progress: 1,
  on_hold: 2,
  scheduled: 3,
  completed: 4,
} satisfies Record<JobStatus, number>;

export const STATUS_BADGE_VARIANT = {
  scheduled: 'neutral',
  in_progress: 'info',
  on_hold: 'warning',
  overdue: 'error',
  completed: 'success',
} satisfies Record<JobStatus, 'neutral' | 'info' | 'warning' | 'error' | 'success'>;

export const PRIORITY_LABEL = {
  urgent: 'Urgent',
  high: 'High',
  normal: 'Normal',
} satisfies Record<JobPriority, string>;

export const PRIORITY_SEVERITY = {
  urgent: 0,
  high: 1,
  normal: 2,
} satisfies Record<JobPriority, number>;

export const PRIORITY_DOT_VARIANT = {
  urgent: 'error',
  high: 'warning',
  normal: 'neutral',
} satisfies Record<JobPriority, 'error' | 'warning' | 'neutral'>;

const quoteFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  maximumFractionDigits: 0,
});

export function formatQuote(amount: number): string {
  return quoteFormatter.format(amount);
}

export function statusLabelOrKey(key: string): string {
  return isJobStatus(key) ? STATUS_LABEL[key] : key;
}

export function priorityLabelOrKey(key: string): string {
  return isJobPriority(key) ? PRIORITY_LABEL[key] : key;
}

export function statusSeverity(key: string): number {
  return isJobStatus(key) ? STATUS_SEVERITY[key] : Number.MAX_SAFE_INTEGER;
}

export function prioritySeverity(key: string): number {
  return isJobPriority(key) ? PRIORITY_SEVERITY[key] : Number.MAX_SAFE_INTEGER;
}

const scheduledFormatter = new Intl.DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  hour: 'numeric',
  minute: '2-digit',
});

export function formatScheduled(iso: string): string {
  return scheduledFormatter.format(new Date(iso));
}
