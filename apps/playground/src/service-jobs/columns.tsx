import { Avatar } from '@astryxdesign/core/Avatar';
import { Badge } from '@astryxdesign/core/Badge';
import { HStack, VStack } from '@astryxdesign/core/Layout';
import { StatusDot } from '@astryxdesign/core/StatusDot';
import { DEFAULT_MIN_COLUMN_WIDTH, pixel, proportional } from '@astryxdesign/core/Table';
import type { ColumnWidth } from '@astryxdesign/core/Table';
import { Text } from '@astryxdesign/core/Text';
import type { ReactNode } from 'react';

import { customerName, technicianName } from '#/service-jobs/data.ts';
import {
  formatQuote,
  formatScheduled,
  PRIORITY_DOT_VARIANT,
  PRIORITY_LABEL,
  STATUS_BADGE_VARIANT,
  STATUS_LABEL,
} from '#/service-jobs/format.ts';
import type { ColumnKey, Job } from '#/service-jobs/service-jobs-types.ts';

export interface ColumnDef {
  key: ColumnKey;
  label: string;
  width: ColumnWidth;
  align?: 'start' | 'end';
  isAlwaysVisible?: boolean;
  renderCell: (job: Job) => ReactNode;
}

export const COLUMN_DEFS: ColumnDef[] = [
  {
    key: 'job',
    label: 'Job',
    width: proportional(2),
    isAlwaysVisible: true,
    renderCell: (job) => (
      <VStack gap={0}>
        <Text type="body" maxLines={1}>
          {job.title}
        </Text>
        <Text type="supporting" color="secondary">
          {job.jobNumber}
        </Text>
      </VStack>
    ),
  },
  {
    key: 'technician',
    label: 'Technician',
    width: proportional(1),
    renderCell: (job) => (
      <HStack gap={2} vAlign="center">
        <Avatar name={technicianName(job.technicianId)} size="sm" />
        <Text type="body" maxLines={1}>
          {technicianName(job.technicianId)}
        </Text>
      </HStack>
    ),
  },
  {
    key: 'customer',
    label: 'Customer',
    width: proportional(1),
    renderCell: (job) => (
      <Text type="body" maxLines={1}>
        {customerName(job.customerId)}
      </Text>
    ),
  },
  {
    key: 'status',
    label: 'Status',
    width: pixel(140),
    renderCell: (job) => (
      <Badge variant={STATUS_BADGE_VARIANT[job.status]} label={STATUS_LABEL[job.status]} />
    ),
  },
  {
    key: 'priority',
    label: 'Priority',
    width: pixel(120),
    renderCell: (job) => (
      <HStack gap={2} vAlign="center">
        <StatusDot
          variant={PRIORITY_DOT_VARIANT[job.priority]}
          label={PRIORITY_LABEL[job.priority]}
        />
        <Text type="body">{PRIORITY_LABEL[job.priority]}</Text>
      </HStack>
    ),
  },
  {
    key: 'scheduled',
    label: 'Scheduled',
    width: pixel(150),
    renderCell: (job) => (
      <Text type="body" color="secondary">
        {formatScheduled(job.scheduledAt)}
      </Text>
    ),
  },
  {
    key: 'quote',
    label: 'Quote',
    width: pixel(110),
    align: 'end',
    renderCell: (job) => <Text type="body">{formatQuote(job.quote)}</Text>,
  },
];

export const DEFAULT_COLUMN_ORDER: ColumnKey[] = COLUMN_DEFS.map((c) => c.key);

export function columnDef(key: ColumnKey): ColumnDef {
  const def = COLUMN_DEFS.find((c) => c.key === key);
  if (!def) throw new Error(`Unknown column: ${key}`);
  return def;
}

/**
 * Pixel columns report their exact width; proportional columns report their guaranteed minimum.
 * Used to estimate sticky-column offsets — exact for the common cases (first start column, any
 * pixel-width end column).
 */
export function estimatedColumnWidth(def: ColumnDef): number {
  return def.width.type === 'pixel' ? def.width.value : DEFAULT_MIN_COLUMN_WIDTH;
}
