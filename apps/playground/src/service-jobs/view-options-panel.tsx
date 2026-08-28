import { CheckboxInput } from '@astryxdesign/core/CheckboxInput';
import { Divider } from '@astryxdesign/core/Divider';
import { Icon } from '@astryxdesign/core/Icon';
import { IconButton } from '@astryxdesign/core/IconButton';
import { HStack, StackItem, VStack } from '@astryxdesign/core/Layout';
import { RadioList, RadioListItem } from '@astryxdesign/core/RadioList';
import { SegmentedControl, SegmentedControlItem } from '@astryxdesign/core/SegmentedControl';
import { Heading, Text } from '@astryxdesign/core/Text';
import { CaretDownIcon, CaretUpIcon, DotsSixVerticalIcon } from '@phosphor-icons/react';
import { useState } from 'react';

import { columnDef } from '#/service-jobs/columns.tsx';
import type { ColumnKey, ViewOptions } from '#/service-jobs/service-jobs-types.ts';
import {
  isDensity,
  isGroupField,
  stickyCountFromString,
} from '#/service-jobs/service-jobs-types.ts';

const GROUP_OPTIONS = [
  { value: 'none', label: 'None' },
  { value: 'status', label: 'Status' },
  { value: 'priority', label: 'Priority' },
  { value: 'technician', label: 'Technician' },
  { value: 'customer', label: 'Customer' },
] as const;

export function ViewOptionsPanel({
  options,
  onChange,
}: {
  options: ViewOptions;
  onChange: (next: ViewOptions) => void;
}) {
  const [draggedKey, setDraggedKey] = useState<ColumnKey | null>(null);
  const activeSet = new Set(options.activeColumns);

  function reorder(nextOrder: ColumnKey[]) {
    onChange({ ...options, columnOrder: nextOrder });
  }

  function moveColumn(key: ColumnKey, direction: -1 | 1) {
    const index = options.columnOrder.indexOf(key);
    const target = index + direction;
    if (target < 0 || target >= options.columnOrder.length) return;
    const nextOrder = [...options.columnOrder];
    const [moved] = nextOrder.splice(index, 1);
    nextOrder.splice(target, 0, moved);
    reorder(nextOrder);
  }

  function dropOn(targetKey: ColumnKey) {
    if (!draggedKey || draggedKey === targetKey) return;
    const nextOrder = options.columnOrder.filter((k) => k !== draggedKey);
    const targetIndex = nextOrder.indexOf(targetKey);
    nextOrder.splice(targetIndex, 0, draggedKey);
    reorder(nextOrder);
  }

  function toggleColumn(key: ColumnKey, checked: boolean) {
    if (key === 'job') return;
    const nextSet = new Set(activeSet);
    if (checked) nextSet.add(key);
    else nextSet.delete(key);
    onChange({
      ...options,
      activeColumns: options.columnOrder.filter((k) => nextSet.has(k)),
    });
  }

  return (
    <VStack gap={5}>
      <VStack gap={3}>
        <Heading level={4}>Columns</Heading>
        <VStack gap={0}>
          {options.columnOrder.map((key, index) => {
            const def = columnDef(key);
            const isAlwaysVisible = Boolean(def.isAlwaysVisible);
            return (
              <HStack
                key={key}
                gap={2}
                vAlign="center"
                draggable
                onDragStart={() => {
                  setDraggedKey(key);
                }}
                onDragOver={(event) => {
                  event.preventDefault();
                }}
                onDrop={(event) => {
                  event.preventDefault();
                  dropOn(key);
                  setDraggedKey(null);
                }}
                onDragEnd={() => {
                  setDraggedKey(null);
                }}
              >
                <Icon icon={DotsSixVerticalIcon} size="sm" color="secondary" />
                <StackItem size="fill">
                  <CheckboxInput
                    label={def.label}
                    description={isAlwaysVisible ? 'Always shown' : undefined}
                    value={activeSet.has(key) || isAlwaysVisible}
                    isDisabled={isAlwaysVisible}
                    onChange={(checked) => {
                      toggleColumn(key, checked);
                    }}
                  />
                </StackItem>
                <IconButton
                  label={`Move ${def.label} up`}
                  variant="ghost"
                  size="sm"
                  icon={<Icon icon={CaretUpIcon} size="sm" />}
                  isDisabled={index === 0}
                  onClick={() => {
                    moveColumn(key, -1);
                  }}
                />
                <IconButton
                  label={`Move ${def.label} down`}
                  variant="ghost"
                  size="sm"
                  icon={<Icon icon={CaretDownIcon} size="sm" />}
                  isDisabled={index === options.columnOrder.length - 1}
                  onClick={() => {
                    moveColumn(key, 1);
                  }}
                />
              </HStack>
            );
          })}
        </VStack>
      </VStack>

      <Divider />

      <VStack gap={3}>
        <Heading level={4}>Density</Heading>
        <SegmentedControl
          label="Row density"
          value={options.density}
          onChange={(value) => {
            if (isDensity(value)) onChange({ ...options, density: value });
          }}
        >
          <SegmentedControlItem value="compact" label="Compact" />
          <SegmentedControlItem value="balanced" label="Balanced" />
          <SegmentedControlItem value="spacious" label="Spacious" />
        </SegmentedControl>
      </VStack>

      <Divider />

      <VStack gap={3}>
        <Heading level={4}>Sticky columns</Heading>
        <HStack gap={4}>
          <VStack gap={2}>
            <Text type="supporting" color="secondary">
              Freeze from start
            </Text>
            <SegmentedControl
              label="Columns frozen from the start"
              value={String(options.stickyStart)}
              onChange={(value) => {
                onChange({ ...options, stickyStart: stickyCountFromString(value) });
              }}
            >
              <SegmentedControlItem value="0" label="None" />
              <SegmentedControlItem value="1" label="1" />
              <SegmentedControlItem value="2" label="2" />
            </SegmentedControl>
          </VStack>
          <VStack gap={2}>
            <Text type="supporting" color="secondary">
              Freeze from end
            </Text>
            <SegmentedControl
              label="Columns frozen from the end"
              value={String(options.stickyEnd)}
              onChange={(value) => {
                onChange({ ...options, stickyEnd: stickyCountFromString(value) });
              }}
            >
              <SegmentedControlItem value="0" label="None" />
              <SegmentedControlItem value="1" label="1" />
              <SegmentedControlItem value="2" label="2" />
            </SegmentedControl>
          </VStack>
        </HStack>
      </VStack>

      <Divider />

      <VStack gap={3}>
        <Heading level={4}>Grouping</Heading>
        <RadioList
          label="Group rows by"
          isLabelHidden
          value={options.grouping}
          onChange={(value) => {
            if (isGroupField(value)) onChange({ ...options, grouping: value });
          }}
        >
          {GROUP_OPTIONS.map((option) => (
            <RadioListItem key={option.value} value={option.value} label={option.label} />
          ))}
        </RadioList>
      </VStack>
    </VStack>
  );
}
