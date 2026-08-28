import { Badge } from '@astryxdesign/core/Badge';
import { Button } from '@astryxdesign/core/Button';
import { DateTimeInput } from '@astryxdesign/core/DateTimeInput';
import type { ISODateTimeString } from '@astryxdesign/core/DateTimeInput';
import { Dialog, DialogHeader } from '@astryxdesign/core/Dialog';
import { HStack, Layout, LayoutContent, LayoutFooter, VStack } from '@astryxdesign/core/Layout';
import { Popover } from '@astryxdesign/core/Popover';
import { RadioList, RadioListItem } from '@astryxdesign/core/RadioList';
import { Text } from '@astryxdesign/core/Text';
import { Toolbar } from '@astryxdesign/core/Toolbar';
import { useState } from 'react';

import { TECHNICIANS } from '#/service-jobs/data.ts';
import type { Job } from '#/service-jobs/service-jobs-types.ts';

export function BulkActionBar({
  selectedJobs,
  onAssign,
  onReschedule,
  onCancelJobs,
  onClearSelection,
}: {
  selectedJobs: Job[];
  onAssign: (technicianId: string) => void;
  onReschedule: (iso: string) => void;
  onCancelJobs: () => void;
  onClearSelection: () => void;
}) {
  const [assignTechnician, setAssignTechnician] = useState('');
  const [rescheduleAt, setRescheduleAt] = useState<ISODateTimeString | undefined>();
  const [isCancelDialogOpen, setIsCancelDialogOpen] = useState(false);
  const count = selectedJobs.length;
  const plural = count === 1 ? '' : 's';

  return (
    <>
      <Toolbar
        label="Bulk actions"
        variant="muted"
        dividers={['top', 'bottom']}
        startContent={
          <HStack gap={2} vAlign="center">
            <Badge variant="info" label={String(count)} />
            <Text type="body" weight="semibold">
              job{plural} selected
            </Text>
          </HStack>
        }
        endContent={
          <HStack gap={2} vAlign="center">
            <Popover
              label="Assign technician"
              width={260}
              content={
                <VStack gap={3}>
                  <RadioList
                    label="Assign to technician"
                    value={assignTechnician}
                    onChange={setAssignTechnician}
                  >
                    {TECHNICIANS.map((technician) => (
                      <RadioListItem
                        key={technician.id}
                        value={technician.id}
                        label={technician.name}
                      />
                    ))}
                  </RadioList>
                  <Button
                    label={`Assign ${count} job${plural}`}
                    variant="primary"
                    size="md"
                    isDisabled={!assignTechnician}
                    onClick={() => {
                      if (assignTechnician) onAssign(assignTechnician);
                    }}
                  />
                </VStack>
              }
            >
              <Button label="Assign" variant="secondary" size="md" />
            </Popover>

            <Popover
              label="Reschedule jobs"
              width={280}
              content={
                <VStack gap={3}>
                  <DateTimeInput
                    label="New date and time"
                    value={rescheduleAt}
                    onChange={setRescheduleAt}
                  />
                  <Button
                    label={`Reschedule ${count} job${plural}`}
                    variant="primary"
                    size="md"
                    isDisabled={!rescheduleAt}
                    onClick={() => {
                      if (rescheduleAt) onReschedule(rescheduleAt);
                    }}
                  />
                </VStack>
              }
            >
              <Button label="Reschedule" variant="secondary" size="md" />
            </Popover>

            <Button
              label="Cancel"
              variant="destructive"
              size="md"
              onClick={() => {
                setIsCancelDialogOpen(true);
              }}
            />
            <Button label="Clear selection" variant="ghost" size="md" onClick={onClearSelection} />
          </HStack>
        }
      />

      <Dialog
        isOpen={isCancelDialogOpen}
        onOpenChange={setIsCancelDialogOpen}
        purpose="required"
        width={440}
      >
        <Layout
          header={
            <DialogHeader
              title={`Cancel ${count} job${plural}?`}
              onOpenChange={setIsCancelDialogOpen}
            />
          }
          content={
            <LayoutContent padding={4}>
              <Text type="body">
                This removes {count === 1 ? 'this job' : `these ${count} jobs`} from the schedule.
                Assigned technicians will need to be notified separately. This cannot be undone.
              </Text>
            </LayoutContent>
          }
          footer={
            <LayoutFooter hasDivider>
              <HStack gap={2} hAlign="end">
                <Button
                  label="Keep jobs"
                  variant="secondary"
                  size="md"
                  onClick={() => {
                    setIsCancelDialogOpen(false);
                  }}
                />
                <Button
                  label={`Cancel ${count} job${plural}`}
                  variant="destructive"
                  size="md"
                  onClick={() => {
                    onCancelJobs();
                    setIsCancelDialogOpen(false);
                  }}
                />
              </HStack>
            </LayoutFooter>
          }
        />
      </Dialog>
    </>
  );
}
