import type {
  EnumItem,
  PowerSearchConfig,
  PowerSearchFilter,
} from '@astryxdesign/core/PowerSearch';

import { CUSTOMERS, TECHNICIANS } from '#/service-jobs/data.ts';
import { formatQuote, PRIORITY_LABEL, STATUS_LABEL } from '#/service-jobs/format.ts';
import type { Job } from '#/service-jobs/service-jobs-types.ts';
import { JOB_PRIORITIES, JOB_STATUSES } from '#/service-jobs/service-jobs-types.ts';

const STATUS_ENUM_ITEMS: EnumItem[] = JOB_STATUSES.map((value) => ({
  value,
  label: STATUS_LABEL[value],
}));

const PRIORITY_ENUM_ITEMS: EnumItem[] = JOB_PRIORITIES.map((value) => ({
  value,
  label: PRIORITY_LABEL[value],
}));

const CUSTOMER_ENUM_ITEMS: EnumItem[] = CUSTOMERS.map((c) => ({ value: c.id, label: c.name }));
const TECHNICIAN_ENUM_ITEMS: EnumItem[] = TECHNICIANS.map((t) => ({ value: t.id, label: t.name }));

export const POWER_SEARCH_CONFIG: PowerSearchConfig = {
  name: 'ServiceJobsSearch',
  contentSearchFieldKey: 'title',
  fields: [
    {
      key: 'title',
      label: 'Job name',
      operators: [{ key: 'contains', label: 'contains', value: { type: 'string' } }],
    },
    {
      key: 'upcoming',
      label: 'Upcoming',
      operators: [{ key: 'is', label: 'is scheduled from now on', value: { type: 'empty' } }],
    },
    {
      key: 'status',
      label: 'Status',
      operators: [
        { key: 'is', label: 'is', value: { type: 'enum', values: STATUS_ENUM_ITEMS } },
        {
          key: 'isAnyOf',
          label: 'is any of',
          value: { type: 'enum_list', values: STATUS_ENUM_ITEMS },
        },
      ],
    },
    {
      key: 'customer',
      label: 'Customer',
      operators: [
        { key: 'is', label: 'is', value: { type: 'enum', values: CUSTOMER_ENUM_ITEMS } },
        {
          key: 'isAnyOf',
          label: 'is any of',
          value: { type: 'enum_list', values: CUSTOMER_ENUM_ITEMS },
        },
      ],
    },
    {
      key: 'technician',
      label: 'Technician',
      operators: [
        { key: 'is', label: 'is', value: { type: 'enum', values: TECHNICIAN_ENUM_ITEMS } },
        {
          key: 'isAnyOf',
          label: 'is any of',
          value: { type: 'enum_list', values: TECHNICIAN_ENUM_ITEMS },
        },
      ],
    },
    {
      key: 'priority',
      label: 'Priority',
      operators: [
        { key: 'is', label: 'is', value: { type: 'enum', values: PRIORITY_ENUM_ITEMS } },
        {
          key: 'isAnyOf',
          label: 'is any of',
          value: { type: 'enum_list', values: PRIORITY_ENUM_ITEMS },
        },
      ],
    },
    {
      key: 'quote',
      label: 'Quote',
      operators: [
        { key: 'gte', label: 'is at least', value: { type: 'float', minValue: 0 } },
        { key: 'lte', label: 'is at most', value: { type: 'float', minValue: 0 } },
      ],
    },
  ],
};

export function fieldFilters(
  filters: readonly PowerSearchFilter[],
  field: string,
): PowerSearchFilter[] {
  return filters.filter((f) => f.field === field);
}

export function withoutField(
  filters: readonly PowerSearchFilter[],
  field: string,
): PowerSearchFilter[] {
  return filters.filter((f) => f.field !== field);
}

export function setSearchText(
  filters: readonly PowerSearchFilter[],
  text: string,
): PowerSearchFilter[] {
  const next = withoutField(filters, 'title');
  const trimmed = text.trim();
  if (!trimmed) return next;
  return [
    ...next,
    { field: 'title', operator: 'contains', value: { type: 'string', value: trimmed } },
  ];
}

export function searchText(filters: readonly PowerSearchFilter[]): string {
  const tokens = fieldFilters(filters, 'title');
  if (tokens.length === 0) return '';
  const [token] = tokens;
  return token.value.type === 'string' ? token.value.value : '';
}

export function toggleUpcoming(filters: readonly PowerSearchFilter[]): PowerSearchFilter[] {
  if (fieldFilters(filters, 'upcoming').length > 0) return withoutField(filters, 'upcoming');
  return [...filters, { field: 'upcoming', operator: 'is', value: { type: 'empty' } }];
}

export function isUpcomingActive(filters: readonly PowerSearchFilter[]): boolean {
  return fieldFilters(filters, 'upcoming').length > 0;
}

/** Replace every token for `field` with a single equality token (the simple-mode chip editor). */
export function setEnumFilter(
  filters: readonly PowerSearchFilter[],
  field: string,
  value: string,
): PowerSearchFilter[] {
  return [
    ...withoutField(filters, field),
    { field, operator: 'is', value: { type: 'enum', value } },
  ];
}

export function setQuoteRange(
  filters: readonly PowerSearchFilter[],
  min: number | null,
  max: number | null,
): PowerSearchFilter[] {
  const next = withoutField(filters, 'quote');
  if (min != null)
    next.push({ field: 'quote', operator: 'gte', value: { type: 'float', value: min } });
  if (max != null)
    next.push({ field: 'quote', operator: 'lte', value: { type: 'float', value: max } });
  return next;
}

export function quoteRange(filters: readonly PowerSearchFilter[]) {
  const tokens = fieldFilters(filters, 'quote');
  const gte = tokens.find((f) => f.operator === 'gte');
  const lte = tokens.find((f) => f.operator === 'lte');
  return {
    min: gte && gte.value.type === 'float' ? gte.value.value : null,
    max: lte && lte.value.type === 'float' ? lte.value.value : null,
  };
}

export function formatQuoteRangeLabel(min: number | null, max: number | null): string {
  if (min != null && max != null) return `${formatQuote(min)}–${formatQuote(max)}`;
  if (min != null) return `≥ ${formatQuote(min)}`;
  if (max != null) return `≤ ${formatQuote(max)}`;
  return '';
}

/** Whether a field's current tokens are simple enough to show as one "field is value" chip. */
export function simpleEnumValue(
  filters: readonly PowerSearchFilter[],
  field: string,
): string | null {
  const tokens = fieldFilters(filters, field);
  if (tokens.length !== 1) return null;
  const [token] = tokens;
  if (token.operator !== 'is' || token.value.type !== 'enum') return null;
  return token.value.value;
}

export function isFieldComplex(filters: readonly PowerSearchFilter[], field: string): boolean {
  const tokens = fieldFilters(filters, field);
  if (tokens.length === 0) return false;
  return simpleEnumValue(filters, field) === null;
}

export function fieldSummary(filters: readonly PowerSearchFilter[], field: string): string | null {
  const tokens = fieldFilters(filters, field);
  if (tokens.length === 0) return null;
  const enumListToken = tokens.find((f) => f.value.type === 'enum_list');
  if (enumListToken && enumListToken.value.type === 'enum_list') {
    return `is any of (${enumListToken.value.value.length})`;
  }
  return `${tokens.length} condition${tokens.length === 1 ? '' : 's'}`;
}

function matchesEnumField(
  filters: readonly PowerSearchFilter[],
  field: string,
  actual: string,
): boolean {
  const tokens = fieldFilters(filters, field);
  if (tokens.length === 0) return true;
  return tokens.some((token) => {
    if (token.value.type === 'enum') return token.value.value === actual;
    if (token.value.type === 'enum_list') return token.value.value.includes(actual);
    return true;
  });
}

export function applyFilters(
  jobs: Job[],
  filters: readonly PowerSearchFilter[],
  now: number,
): Job[] {
  const titleTokens = fieldFilters(filters, 'title');
  const quoteTokens = fieldFilters(filters, 'quote');
  const upcoming = isUpcomingActive(filters);

  return jobs.filter((job) => {
    if (
      titleTokens.length > 0 &&
      !titleTokens.some(
        (token) =>
          token.value.type === 'string' &&
          (job.title.toLowerCase().includes(token.value.value.toLowerCase()) ||
            job.jobNumber.toLowerCase().includes(token.value.value.toLowerCase())),
      )
    ) {
      return false;
    }
    if (upcoming && new Date(job.scheduledAt).getTime() < now) return false;
    if (!matchesEnumField(filters, 'status', job.status)) return false;
    if (!matchesEnumField(filters, 'customer', job.customerId)) return false;
    if (!matchesEnumField(filters, 'technician', job.technicianId)) return false;
    if (!matchesEnumField(filters, 'priority', job.priority)) return false;
    for (const token of quoteTokens) {
      if (token.value.type !== 'float') continue;
      if (token.operator === 'gte' && job.quote < token.value.value) return false;
      if (token.operator === 'lte' && job.quote > token.value.value) return false;
    }
    return true;
  });
}
