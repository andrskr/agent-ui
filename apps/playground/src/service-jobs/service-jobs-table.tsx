/* eslint-disable oxlint-plugin/no-style-escape-hatches --
 * Sticky-column offsets are per-column pixel values computed at render time from the active
 * column order and sticky-edge counts; StyleX classes are compiled statically and cannot express
 * a runtime-computed offset. See references/coverage-gaps.md ("Design-system gaps") for the
 * recorded gap: the design system has no data-driven sticky-columns story for children-mode Table.
 */
import { Badge } from '@astryxdesign/core/Badge';
import { Center } from '@astryxdesign/core/Center';
import { CheckboxInput } from '@astryxdesign/core/CheckboxInput';
import { EmptyState } from '@astryxdesign/core/EmptyState';
import { Icon } from '@astryxdesign/core/Icon';
import { IconButton } from '@astryxdesign/core/IconButton';
import { HStack } from '@astryxdesign/core/Layout';
import { Skeleton } from '@astryxdesign/core/Skeleton';
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableHeaderCell,
  TableRow,
} from '@astryxdesign/core/Table';
import { Text } from '@astryxdesign/core/Text';
import { colorVars } from '@astryxdesign/core/theme/tokens.stylex';
import {
  ArrowDownIcon,
  ArrowsDownUpIcon,
  ArrowUpIcon,
  ArrowSquareOutIcon,
  CaretDownIcon,
  CaretRightIcon,
} from '@phosphor-icons/react';
import * as stylex from '@stylexjs/stylex';
import type { CSSProperties, KeyboardEvent, MouseEvent } from 'react';
import { Fragment } from 'react';

import { estimatedColumnWidth, type ColumnDef } from '#/service-jobs/columns.tsx';
import { groupKeyFor, groupLabelFor } from '#/service-jobs/grouping.ts';
import type { Density, GroupField, Job, SortState } from '#/service-jobs/service-jobs-types.ts';

const CHECKBOX_COLUMN_WIDTH = 44;
const OPEN_COLUMN_WIDTH = 48;

const styles = stylex.create({
  groupHeaderRow: {
    cursor: 'pointer',
  },
  groupHeaderCell: {
    backgroundColor: colorVars['--color-background-muted'],
  },
});

function stickyStyle(
  offset: number | undefined,
  side: 'left' | 'right',
): CSSProperties | undefined {
  if (offset === undefined) return undefined;
  return {
    position: 'sticky',
    [side]: offset,
    zIndex: 1,
    backgroundColor: colorVars['--color-background-surface'],
  };
}

function computeOffsets(activeDefs: ColumnDef[], stickyStart: number, stickyEnd: number) {
  const startOffsets = new Map<string, number>();
  let runningStart = CHECKBOX_COLUMN_WIDTH;
  for (let i = 0; i < Math.min(stickyStart, activeDefs.length); i++) {
    startOffsets.set(activeDefs[i].key, runningStart);
    runningStart += estimatedColumnWidth(activeDefs[i]);
  }
  const endOffsets = new Map<string, number>();
  let runningEnd = OPEN_COLUMN_WIDTH;
  const remaining = activeDefs.length - stickyStart;
  const clampedEnd = Math.min(stickyEnd, Math.max(remaining, 0));
  for (let i = 0; i < clampedEnd; i++) {
    const def = activeDefs[activeDefs.length - 1 - i];
    endOffsets.set(def.key, runningEnd);
    runningEnd += estimatedColumnWidth(def);
  }
  return { startOffsets, endOffsets };
}

export function ServiceJobsTable({
  activeColumnDefs,
  density,
  stickyStart,
  stickyEnd,
  jobs,
  grouping,
  groupOrder,
  collapsedGroups,
  onToggleGroup,
  sort,
  onSortChange,
  selectedIds,
  onToggleSelect,
  onToggleSelectAll,
  onOpenJob,
  hasMore,
  isLoadingMore,
  isInitialLoading,
}: {
  activeColumnDefs: ColumnDef[];
  density: Density;
  stickyStart: 0 | 1 | 2;
  stickyEnd: 0 | 1 | 2;
  jobs: Job[];
  grouping: GroupField;
  groupOrder: string[];
  collapsedGroups: Set<string>;
  onToggleGroup: (key: string) => void;
  sort: SortState;
  onSortChange: (next: SortState) => void;
  selectedIds: Set<string>;
  onToggleSelect: (jobId: string, isSelected: boolean) => void;
  onToggleSelectAll: (isSelected: boolean) => void;
  onOpenJob: (jobId: string) => void;
  hasMore: boolean;
  isLoadingMore: boolean;
  isInitialLoading: boolean;
}) {
  const { startOffsets, endOffsets } = computeOffsets(activeColumnDefs, stickyStart, stickyEnd);
  const columnCount = activeColumnDefs.length + 2;
  const isAllSelected = jobs.length > 0 && jobs.every((job) => selectedIds.has(job.id));
  const isIndeterminate = !isAllSelected && jobs.some((job) => selectedIds.has(job.id));

  const grouped = new Map<string, Job[]>();
  if (grouping !== 'none') {
    for (const job of jobs) {
      const key = groupKeyFor(job, grouping);
      const bucket = grouped.get(key);
      if (bucket) bucket.push(job);
      else grouped.set(key, [job]);
    }
  }

  if (isInitialLoading) {
    return (
      <Table columns={[]} density={density} dividers="rows">
        <TableBody>
          {Array.from({ length: 8 }, (_, index) => (
            <TableRow key={index}>
              <TableCell colSpan={columnCount}>
                <Skeleton height={20} index={index} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }

  if (jobs.length === 0) {
    return (
      <Center>
        <EmptyState
          title="No jobs match your filters"
          description="Try adjusting or clearing your filters to see more results."
        />
      </Center>
    );
  }

  function renderRow(job: Job) {
    return (
      <TableRow
        key={job.id}
        onClick={() => {
          onOpenJob(job.id);
        }}
      >
        <TableCell style={stickyStyle(0, 'left')}>
          <Center
            axis="horizontal"
            onClick={(event: MouseEvent) => {
              event.stopPropagation();
            }}
          >
            <CheckboxInput
              label={`Select ${job.title}`}
              isLabelHidden
              value={selectedIds.has(job.id)}
              onChange={(checked) => {
                onToggleSelect(job.id, checked);
              }}
            />
          </Center>
        </TableCell>
        {activeColumnDefs.map((def) => (
          <TableCell
            key={def.key}
            style={
              stickyStyle(startOffsets.get(def.key), 'left') ??
              stickyStyle(endOffsets.get(def.key), 'right')
            }
          >
            {def.renderCell(job)}
          </TableCell>
        ))}
        <TableCell style={stickyStyle(0, 'right')}>
          <Center
            axis="horizontal"
            onClick={(event: MouseEvent) => {
              event.stopPropagation();
            }}
          >
            <IconButton
              label={`Open details for ${job.title}`}
              variant="ghost"
              size="sm"
              icon={<Icon icon={ArrowSquareOutIcon} size="sm" />}
              onClick={() => {
                onOpenJob(job.id);
              }}
            />
          </Center>
        </TableCell>
      </TableRow>
    );
  }

  function sortIndicator(key: ColumnDef['key']) {
    if (sort.columnKey !== key) return <Icon icon={ArrowsDownUpIcon} size="sm" color="secondary" />;
    return (
      <Icon
        icon={sort.direction === 'ascending' ? ArrowUpIcon : ArrowDownIcon}
        size="sm"
        color="accent"
      />
    );
  }

  function toggleSort(key: ColumnDef['key']) {
    if (sort.columnKey !== key) {
      onSortChange({ columnKey: key, direction: 'ascending' });
      return;
    }
    onSortChange({
      columnKey: key,
      direction: sort.direction === 'ascending' ? 'descending' : 'ascending',
    });
  }

  return (
    <Table columns={[]} density={density} dividers="rows" hasHover textOverflow="truncate">
      <TableHeader>
        <TableRow>
          <TableHeaderCell style={stickyStyle(0, 'left')}>
            <Center axis="horizontal">
              <CheckboxInput
                label="Select all loaded jobs"
                isLabelHidden
                value={isIndeterminate ? 'indeterminate' : isAllSelected}
                onChange={(checked) => {
                  onToggleSelectAll(checked);
                }}
              />
            </Center>
          </TableHeaderCell>
          {activeColumnDefs.map((def) => (
            <TableHeaderCell
              key={def.key}
              style={
                stickyStyle(startOffsets.get(def.key), 'left') ??
                stickyStyle(endOffsets.get(def.key), 'right')
              }
            >
              <HStack
                as="button"
                gap={1}
                vAlign="center"
                onClick={() => {
                  toggleSort(def.key);
                }}
              >
                <Text type="body" weight="semibold">
                  {def.label}
                </Text>
                {sortIndicator(def.key)}
              </HStack>
            </TableHeaderCell>
          ))}
          <TableHeaderCell style={stickyStyle(0, 'right')} aria-label="Open job details" />
        </TableRow>
      </TableHeader>
      <TableBody>
        {grouping === 'none'
          ? jobs.map((job) => renderRow(job))
          : groupOrder.map((key) => {
              const bucket = grouped.get(key);
              if (!bucket || bucket.length === 0) return null;
              const isCollapsed = collapsedGroups.has(key);
              return (
                <Fragment key={key}>
                  <TableRow
                    xstyle={[styles.groupHeaderRow]}
                    // eslint-disable-next-line jsx-a11y/prefer-tag-over-role -- TableRow always renders <tr>; a collapsible group header has no valid semantic substitute inside a table body.
                    role="button"
                    tabIndex={0}
                    onClick={() => {
                      onToggleGroup(key);
                    }}
                    onKeyDown={(event: KeyboardEvent) => {
                      if (event.key === 'Enter' || event.key === ' ') {
                        event.preventDefault();
                        onToggleGroup(key);
                      }
                    }}
                  >
                    <TableCell colSpan={columnCount} xstyle={[styles.groupHeaderCell]}>
                      <HStack gap={2} vAlign="center">
                        <Icon
                          icon={isCollapsed ? CaretRightIcon : CaretDownIcon}
                          size="sm"
                          color="secondary"
                        />
                        <Text type="body" weight="bold">
                          {groupLabelFor(grouping, key)}
                        </Text>
                        <Badge variant="neutral" label={String(bucket.length)} />
                      </HStack>
                    </TableCell>
                  </TableRow>
                  {!isCollapsed && bucket.map((job) => renderRow(job))}
                </Fragment>
              );
            })}
        {hasMore && (
          <TableRow>
            <TableCell colSpan={columnCount}>
              <Center>
                {isLoadingMore ? (
                  <Text type="supporting" color="secondary">
                    Loading more jobs…
                  </Text>
                ) : (
                  <Text type="supporting" color="secondary">
                    Scroll for more
                  </Text>
                )}
              </Center>
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
