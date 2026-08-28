import { Button } from '@astryxdesign/core/Button';
import { DateTimeInput } from '@astryxdesign/core/DateTimeInput';
import type { ISODateTimeString } from '@astryxdesign/core/DateTimeInput';
import { Dialog, DialogHeader } from '@astryxdesign/core/Dialog';
import { HStack, Layout, LayoutContent, LayoutFooter, VStack } from '@astryxdesign/core/Layout';
import { NumberInput } from '@astryxdesign/core/NumberInput';
import { Selector } from '@astryxdesign/core/Selector';
import { TextInput } from '@astryxdesign/core/TextInput';
import { useState } from 'react';

import { CUSTOMERS, TECHNICIANS } from '#/service-jobs/data.ts';
import { PRIORITY_LABEL } from '#/service-jobs/format.ts';
import type { Job, JobPriority } from '#/service-jobs/service-jobs-types.ts';
import { isJobPriority, JOB_PRIORITIES } from '#/service-jobs/service-jobs-types.ts';

const PRIORITY_OPTIONS = JOB_PRIORITIES.map((value) => ({
  value,
  label: PRIORITY_LABEL[value],
}));

function makeJobId(): string {
  return `job-new-${CUSTOMERS.length}-${Date.now()}`;
}

export function NewJobDialog({
  isOpen,
  onOpenChange,
  onCreate,
}: {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
  onCreate: (job: Job) => void;
}) {
  const [title, setTitle] = useState('');
  const [customerId, setCustomerId] = useState(CUSTOMERS[0].id);
  const [technicianId, setTechnicianId] = useState(TECHNICIANS[0].id);
  const [priority, setPriority] = useState<JobPriority>('normal');
  const [scheduledAt, setScheduledAt] = useState<ISODateTimeString | undefined>();
  const [quote, setQuote] = useState<number | null>(null);

  const isValid = title.trim().length > 0 && scheduledAt !== undefined && quote !== null;

  function reset() {
    setTitle('');
    setCustomerId(CUSTOMERS[0].id);
    setTechnicianId(TECHNICIANS[0].id);
    setPriority('normal');
    setScheduledAt(undefined);
    setQuote(null);
  }

  function handleCreate() {
    if (!isValid) return;
    onCreate({
      id: makeJobId(),
      jobNumber: `SJ-${Math.floor(10_000 + Math.random() * 89_999)}`,
      title: title.trim(),
      customerId,
      technicianId,
      status: 'scheduled',
      priority,
      scheduledAt: new Date(scheduledAt).toISOString(),
      quote,
    });
    reset();
    onOpenChange(false);
  }

  return (
    <Dialog
      isOpen={isOpen}
      onOpenChange={(next) => {
        if (!next) reset();
        onOpenChange(next);
      }}
      purpose="form"
      width={480}
    >
      <Layout
        header={<DialogHeader title="New job" onOpenChange={onOpenChange} />}
        content={
          <LayoutContent padding={4}>
            <VStack gap={4}>
              <TextInput label="Job name" value={title} onChange={setTitle} isRequired />
              <Selector
                label="Customer"
                value={customerId}
                onChange={setCustomerId}
                options={CUSTOMERS.map((c) => ({ value: c.id, label: c.name }))}
              />
              <Selector
                label="Technician"
                value={technicianId}
                onChange={setTechnicianId}
                options={TECHNICIANS.map((t) => ({ value: t.id, label: t.name }))}
              />
              <Selector
                label="Priority"
                value={priority}
                onChange={(value) => {
                  if (isJobPriority(value)) setPriority(value);
                }}
                options={PRIORITY_OPTIONS}
              />
              <DateTimeInput
                label="Scheduled for"
                value={scheduledAt}
                onChange={setScheduledAt}
                isRequired
              />
              <NumberInput
                label="Quote"
                value={quote}
                onChange={setQuote}
                min={0}
                units="USD"
                isRequired
              />
            </VStack>
          </LayoutContent>
        }
        footer={
          <LayoutFooter hasDivider>
            <HStack gap={2} hAlign="end">
              <Button
                label="Cancel"
                variant="secondary"
                size="md"
                onClick={() => {
                  onOpenChange(false);
                }}
              />
              <Button
                label="Create job"
                variant="primary"
                size="md"
                isDisabled={!isValid}
                onClick={handleCreate}
              />
            </HStack>
          </LayoutFooter>
        }
      />
    </Dialog>
  );
}
