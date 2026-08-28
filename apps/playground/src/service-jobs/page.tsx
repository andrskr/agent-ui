import { Button } from '@astryxdesign/core/Button';
import {
  HStack,
  Layout,
  LayoutContent,
  LayoutHeader,
  StackItem,
  VStack,
} from '@astryxdesign/core/Layout';
import { Popover } from '@astryxdesign/core/Popover';
import type { PowerSearchFilter } from '@astryxdesign/core/PowerSearch';
import { useResizable, ResizeHandle } from '@astryxdesign/core/Resizable';
import { Heading } from '@astryxdesign/core/Text';
import { PlusIcon } from '@phosphor-icons/react';
import { useMemo, useState } from 'react';

import { BulkActionBar } from '#/service-jobs/bulk-action-bar.tsx';
import { columnDef } from '#/service-jobs/columns.tsx';
import { customerName, JOBS, technicianName } from '#/service-jobs/data.ts';
import { JobDetailPanel } from '#/service-jobs/detail-panel.tsx';
import { FilterBar } from '#/service-jobs/filter-bar.tsx';
import { applyFilters } from '#/service-jobs/filters.ts';
import { PRIORITY_SEVERITY, STATUS_SEVERITY } from '#/service-jobs/format.ts';
import { groupOrderFor } from '#/service-jobs/grouping.ts';
import { NewJobDialog } from '#/service-jobs/new-job-dialog.tsx';
import { SavedViewsMenu } from '#/service-jobs/saved-views-menu.tsx';
import { ServiceJobsTable } from '#/service-jobs/service-jobs-table.tsx';
import type { Job, SavedView, SortState } from '#/service-jobs/service-jobs-types.ts';
import { useInfiniteJobs } from '#/service-jobs/use-infinite-jobs.ts';
import { BUILT_IN_SAVED_VIEWS, DEFAULT_VIEW_STATE } from '#/service-jobs/view-defaults.ts';
import { ViewOptionsPanel } from '#/service-jobs/view-options-panel.tsx';

function compareJobs(a: Job, b: Job, sort: SortState): number {
  const dir = sort.direction === 'ascending' ? 1 : -1;
  switch (sort.columnKey) {
    case 'job':
      return dir * a.title.localeCompare(b.title);
    case 'technician':
      return dir * technicianName(a.technicianId).localeCompare(technicianName(b.technicianId));
    case 'customer':
      return dir * customerName(a.customerId).localeCompare(customerName(b.customerId));
    case 'status':
      return dir * (STATUS_SEVERITY[a.status] - STATUS_SEVERITY[b.status]);
    case 'priority':
      return dir * (PRIORITY_SEVERITY[a.priority] - PRIORITY_SEVERITY[b.priority]);
    case 'scheduled':
      return dir * (new Date(a.scheduledAt).getTime() - new Date(b.scheduledAt).getTime());
    case 'quote':
      return dir * (a.quote - b.quote);
    default:
      return 0;
  }
}

export default function ServiceJobsPage() {
  const [jobs, setJobs] = useState<Job[]>(JOBS);
  const [filters, setFilters] = useState<PowerSearchFilter[]>(DEFAULT_VIEW_STATE.filters);
  const [sort, setSort] = useState<SortState>(DEFAULT_VIEW_STATE.sort);
  const [options, setOptions] = useState(DEFAULT_VIEW_STATE.options);
  const [savedViews, setSavedViews] = useState<SavedView[]>(BUILT_IN_SAVED_VIEWS);
  const [activeViewId, setActiveViewId] = useState<string | null>(null);

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
  const [openJobId, setOpenJobId] = useState<string | null>(null);
  const [isViewOptionsOpen, setIsViewOptionsOpen] = useState(false);
  const [isNewJobDialogOpen, setIsNewJobDialogOpen] = useState(false);

  const detailPanel = useResizable({ defaultSize: 380, minSizePx: 300, maxSizePx: 560 });
  const [now] = useState(() => Date.now());

  const filteredJobs = useMemo(() => applyFilters(jobs, filters, now), [jobs, filters, now]);

  const sortedJobs = useMemo(
    // eslint-disable-next-line unicorn/no-array-sort -- toSorted() needs an ES2023 lib target; this project targets ES2022, so copy-then-sort is the compliant equivalent.
    () => [...filteredJobs].sort((a, b) => compareJobs(a, b, sort)),
    [filteredJobs, sort],
  );

  const groupOrder = useMemo(
    () => groupOrderFor(options.grouping, sortedJobs),
    [options.grouping, sortedJobs],
  );

  const resetKey = useMemo(
    () => JSON.stringify({ filters, sort, grouping: options.grouping }),
    [filters, sort, options.grouping],
  );

  const { visibleJobs, hasMore, isLoadingMore, loadMore } = useInfiniteJobs(
    sortedJobs,
    options.grouping,
    resetKey,
  );

  const activeColumnDefs = options.columnOrder
    .filter((key) => options.activeColumns.includes(key))
    .map((key) => columnDef(key));

  const selectedJobs = jobs.filter((job) => selectedIds.has(job.id));
  const openJob = jobs.find((job) => job.id === openJobId) ?? null;

  function toggleSelect(jobId: string, isSelected: boolean) {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (isSelected) next.add(jobId);
      else next.delete(jobId);
      return next;
    });
  }

  function toggleSelectAll(isSelected: boolean) {
    setSelectedIds(isSelected ? new Set(visibleJobs.map((job) => job.id)) : new Set());
  }

  function toggleGroup(key: string) {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  function applyView(view: SavedView) {
    setFilters(view.state.filters);
    setSort(view.state.sort);
    setOptions(view.state.options);
    setActiveViewId(view.id);
    setSelectedIds(new Set());
  }

  function saveView(name: string) {
    const id = `view-${name.toLowerCase().replaceAll(/\s+/g, '-')}-${savedViews.length}`;
    const view: SavedView = { id, name, state: { filters, sort, options } };
    setSavedViews((prev) => [...prev, view]);
    setActiveViewId(id);
  }

  function handleScroll(event: {
    currentTarget: { scrollTop: number; scrollHeight: number; clientHeight: number };
  }) {
    const { scrollTop, scrollHeight, clientHeight } = event.currentTarget;
    if (scrollHeight - scrollTop - clientHeight < 200) loadMore();
  }

  const activeViewName = savedViews.find((view) => view.id === activeViewId)?.name ?? null;

  return (
    <>
      <Layout
        height="fill"
        header={
          <LayoutHeader hasDivider padding={4}>
            <VStack gap={4}>
              <HStack gap={3} vAlign="center">
                <StackItem size="fill">
                  <Heading level={1}>Service jobs</Heading>
                </StackItem>
                <SavedViewsMenu
                  savedViews={savedViews}
                  activeViewName={activeViewName}
                  onApplyView={applyView}
                  onSaveView={saveView}
                />
                <Button
                  label="New job"
                  variant="primary"
                  size="md"
                  icon={<PlusIcon />}
                  onClick={() => {
                    setIsNewJobDialogOpen(true);
                  }}
                />
              </HStack>

              {selectedIds.size > 0 ? (
                <BulkActionBar
                  selectedJobs={selectedJobs}
                  onAssign={(technicianId) => {
                    setJobs((prev) =>
                      prev.map((job) => (selectedIds.has(job.id) ? { ...job, technicianId } : job)),
                    );
                    setSelectedIds(new Set());
                  }}
                  onReschedule={(iso) => {
                    setJobs((prev) =>
                      prev.map((job) =>
                        selectedIds.has(job.id) ? { ...job, scheduledAt: iso } : job,
                      ),
                    );
                    setSelectedIds(new Set());
                  }}
                  onCancelJobs={() => {
                    setJobs((prev) => prev.filter((job) => !selectedIds.has(job.id)));
                    if (openJobId !== null && selectedIds.has(openJobId)) setOpenJobId(null);
                    setSelectedIds(new Set());
                  }}
                  onClearSelection={() => {
                    setSelectedIds(new Set());
                  }}
                />
              ) : (
                <Popover
                  isOpen={isViewOptionsOpen}
                  onOpenChange={setIsViewOptionsOpen}
                  placement="below"
                  alignment="end"
                  width={340}
                  label="View options"
                  content={<ViewOptionsPanel options={options} onChange={setOptions} />}
                >
                  <FilterBar
                    filters={filters}
                    onFiltersChange={setFilters}
                    resultCount={filteredJobs.length}
                    onOpenViewOptions={() => {
                      setIsViewOptionsOpen(true);
                    }}
                  />
                </Popover>
              )}
            </VStack>
          </LayoutHeader>
        }
        content={
          <LayoutContent
            padding={0}
            onScroll={(event) => {
              handleScroll(event);
            }}
          >
            <ServiceJobsTable
              activeColumnDefs={activeColumnDefs}
              density={options.density}
              stickyStart={options.stickyStart}
              stickyEnd={options.stickyEnd}
              jobs={visibleJobs}
              grouping={options.grouping}
              groupOrder={groupOrder}
              collapsedGroups={collapsedGroups}
              onToggleGroup={toggleGroup}
              sort={sort}
              onSortChange={setSort}
              selectedIds={selectedIds}
              onToggleSelect={toggleSelect}
              onToggleSelectAll={toggleSelectAll}
              onOpenJob={setOpenJobId}
              hasMore={hasMore}
              isLoadingMore={isLoadingMore}
              isInitialLoading={false}
            />
          </LayoutContent>
        }
        end={
          openJob && (
            <>
              <ResizeHandle resizable={detailPanel.props} isReversed isAlwaysVisible={false} />
              <JobDetailPanel
                job={openJob}
                onClose={() => {
                  setOpenJobId(null);
                }}
                resizable={detailPanel.props}
              />
            </>
          )
        }
      />
      <NewJobDialog
        isOpen={isNewJobDialogOpen}
        onOpenChange={setIsNewJobDialogOpen}
        onCreate={(job) => {
          setJobs((prev) => [job, ...prev]);
        }}
      />
    </>
  );
}
