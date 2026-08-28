import { Avatar } from '@astryxdesign/core/Avatar';
import { Badge } from '@astryxdesign/core/Badge';
import { Button } from '@astryxdesign/core/Button';
import { Icon } from '@astryxdesign/core/Icon';
import { HStack, LayoutPanel, StackItem, VStack } from '@astryxdesign/core/Layout';
import { MetadataList, MetadataListItem } from '@astryxdesign/core/MetadataList';
import type { ResizableProps } from '@astryxdesign/core/Resizable';
import { StatusDot } from '@astryxdesign/core/StatusDot';
import { Heading, Text } from '@astryxdesign/core/Text';
import { XIcon } from '@phosphor-icons/react';

import { customerName, technicianName } from '#/service-jobs/data.ts';
import {
  formatQuote,
  formatScheduled,
  PRIORITY_DOT_VARIANT,
  PRIORITY_LABEL,
  STATUS_BADGE_VARIANT,
  STATUS_LABEL,
} from '#/service-jobs/format.ts';
import type { Job } from '#/service-jobs/service-jobs-types.ts';

export function JobDetailPanel({
  job,
  onClose,
  resizable,
}: {
  job: Job;
  onClose: () => void;
  resizable: ResizableProps;
}) {
  return (
    <LayoutPanel
      hasDivider
      resizable={resizable}
      padding={4}
      // eslint-disable-next-line jsx-a11y/prefer-tag-over-role -- LayoutPanel always renders a div; its docs sanction role="complementary" as the supported landmark API.
      role="complementary"
      label={`Job details — ${job.title}`}
    >
      <VStack gap={4}>
        <HStack gap={2} vAlign="center">
          <StackItem size="fill">
            <Text type="supporting" color="secondary">
              {job.jobNumber}
            </Text>
          </StackItem>
          <Button
            label="Close panel"
            variant="ghost"
            size="sm"
            isIconOnly
            icon={<Icon icon={XIcon} size="sm" />}
            onClick={onClose}
          />
        </HStack>

        <Heading level={3}>{job.title}</Heading>

        <MetadataList label={{ position: 'start' }}>
          <MetadataListItem label="Status">
            <Badge variant={STATUS_BADGE_VARIANT[job.status]} label={STATUS_LABEL[job.status]} />
          </MetadataListItem>
          <MetadataListItem label="Priority">
            <HStack gap={2} vAlign="center">
              <StatusDot
                variant={PRIORITY_DOT_VARIANT[job.priority]}
                label={PRIORITY_LABEL[job.priority]}
              />
              <Text type="body">{PRIORITY_LABEL[job.priority]}</Text>
            </HStack>
          </MetadataListItem>
          <MetadataListItem label="Technician">
            <HStack gap={2} vAlign="center">
              <Avatar name={technicianName(job.technicianId)} size="sm" />
              <Text type="body">{technicianName(job.technicianId)}</Text>
            </HStack>
          </MetadataListItem>
          <MetadataListItem label="Customer">{customerName(job.customerId)}</MetadataListItem>
          <MetadataListItem label="Scheduled">{formatScheduled(job.scheduledAt)}</MetadataListItem>
          <MetadataListItem label="Quote">{formatQuote(job.quote)}</MetadataListItem>
        </MetadataList>
      </VStack>
    </LayoutPanel>
  );
}
