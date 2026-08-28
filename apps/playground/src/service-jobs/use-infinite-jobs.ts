import { useEffect, useMemo, useRef, useState } from 'react';

import { groupKeyFor } from '#/service-jobs/grouping.ts';
import type { GroupField, Job } from '#/service-jobs/service-jobs-types.ts';

const BATCH_SIZE = 40;
const SIMULATED_LATENCY_MS = 500;

function extendToGroupBoundary(count: number, jobs: Job[], groupField: GroupField): number {
  if (groupField === 'none' || count >= jobs.length) return Math.min(count, jobs.length);
  const boundaryKey = groupKeyFor(jobs[count - 1], groupField);
  let idx = count;
  while (idx < jobs.length && groupKeyFor(jobs[idx], groupField) === boundaryKey) idx++;
  return idx;
}

export function useInfiniteJobs(jobs: Job[], groupField: GroupField, resetKey: string) {
  const [loadedCount, setLoadedCount] = useState(() =>
    extendToGroupBoundary(Math.min(BATCH_SIZE, jobs.length), jobs, groupField),
  );
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // A changed filter, sort, or grouping resets back to the first batch.
  useEffect(() => {
    setLoadedCount(extendToGroupBoundary(Math.min(BATCH_SIZE, jobs.length), jobs, groupField));
    setIsLoadingMore(false);
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- intentionally reset only on resetKey (filter/sort/grouping changes), not on every `jobs` mutation from a bulk edit.
  }, [resetKey]);

  useEffect(
    () => () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    },
    [],
  );

  const hasMore = loadedCount < jobs.length;

  function loadMore() {
    if (isLoadingMore || !hasMore) return;
    setIsLoadingMore(true);
    timeoutRef.current = setTimeout(() => {
      setLoadedCount((prev) => extendToGroupBoundary(prev + BATCH_SIZE, jobs, groupField));
      setIsLoadingMore(false);
    }, SIMULATED_LATENCY_MS);
  }

  const visibleJobs = useMemo(() => jobs.slice(0, loadedCount), [jobs, loadedCount]);

  return { visibleJobs, hasMore, isLoadingMore, loadMore, loadedCount };
}
