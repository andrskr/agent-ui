import { Button } from '@astryxdesign/core/Button';
import { Icon } from '@astryxdesign/core/Icon';
import { IconButton } from '@astryxdesign/core/IconButton';
import { HStack, StackItem } from '@astryxdesign/core/Layout';
import { NumberInput } from '@astryxdesign/core/NumberInput';
import { Popover } from '@astryxdesign/core/Popover';
import { PowerSearch } from '@astryxdesign/core/PowerSearch';
import type { PowerSearchFilter } from '@astryxdesign/core/PowerSearch';
import { RadioList, RadioListItem } from '@astryxdesign/core/RadioList';
import { Text } from '@astryxdesign/core/Text';
import { TextInput } from '@astryxdesign/core/TextInput';
import {
  FunnelIcon,
  ListMagnifyingGlassIcon,
  MagnifyingGlassIcon,
  SlidersHorizontalIcon,
  XIcon,
} from '@phosphor-icons/react';
import { useState } from 'react';

import { CUSTOMERS, TECHNICIANS } from '#/service-jobs/data.ts';
import {
  fieldSummary,
  formatQuoteRangeLabel,
  isFieldComplex,
  isUpcomingActive,
  POWER_SEARCH_CONFIG,
  quoteRange,
  searchText,
  setEnumFilter,
  setQuoteRange,
  setSearchText,
  simpleEnumValue,
  toggleUpcoming,
  withoutField,
} from '#/service-jobs/filters.ts';
import { PRIORITY_LABEL, STATUS_LABEL } from '#/service-jobs/format.ts';
import { JOB_PRIORITIES, JOB_STATUSES } from '#/service-jobs/service-jobs-types.ts';

interface FilterOption {
  value: string;
  label: string;
}

function EnumFilterChip({
  label,
  field,
  options,
  filters,
  onChange,
}: {
  label: string;
  field: string;
  options: FilterOption[];
  filters: PowerSearchFilter[];
  onChange: (next: PowerSearchFilter[]) => void;
}) {
  const value = simpleEnumValue(filters, field);
  const complex = isFieldComplex(filters, field);
  const isSet = complex || value !== null;

  function computeTriggerLabel(): string {
    if (complex) return `${label} ${fieldSummary(filters, field) ?? ''}`;
    if (value !== null)
      return `${label} is ${options.find((o) => o.value === value)?.label ?? value}`;
    return label;
  }

  const triggerLabel = computeTriggerLabel();

  return (
    <HStack gap={0} vAlign="center">
      <Popover
        placement="below"
        alignment="start"
        width={240}
        label={`${label} filter`}
        content={
          <RadioList
            label={label}
            isLabelHidden
            value={value ?? ''}
            onChange={(next) => {
              onChange(setEnumFilter(filters, field, next));
            }}
          >
            {options.map((option) => (
              <RadioListItem key={option.value} value={option.value} label={option.label} />
            ))}
          </RadioList>
        }
      >
        <Button label={triggerLabel} variant={isSet ? 'secondary' : 'ghost'} size="md" />
      </Popover>
      {isSet && (
        <IconButton
          label={`Clear ${label} filter`}
          variant="ghost"
          size="sm"
          icon={<Icon icon={XIcon} size="sm" />}
          onClick={() => {
            onChange(withoutField(filters, field));
          }}
        />
      )}
    </HStack>
  );
}

function QuoteFilterChip({
  filters,
  onChange,
}: {
  filters: PowerSearchFilter[];
  onChange: (next: PowerSearchFilter[]) => void;
}) {
  const { min, max } = quoteRange(filters);
  const isSet = min != null || max != null;
  const triggerLabel = isSet ? `Quote ${formatQuoteRangeLabel(min, max)}` : 'Quote';

  return (
    <HStack gap={0} vAlign="center">
      <Popover
        placement="below"
        alignment="start"
        width={220}
        label="Quote filter"
        content={
          <HStack gap={3}>
            <NumberInput
              label="Min"
              value={min}
              min={0}
              hasClear
              onChange={(next) => {
                onChange(setQuoteRange(filters, next, max));
              }}
            />
            <NumberInput
              label="Max"
              value={max}
              min={0}
              hasClear
              onChange={(next) => {
                onChange(setQuoteRange(filters, min, next));
              }}
            />
          </HStack>
        }
      >
        <Button label={triggerLabel} variant={isSet ? 'secondary' : 'ghost'} size="md" />
      </Popover>
      {isSet && (
        <IconButton
          label="Clear Quote filter"
          variant="ghost"
          size="sm"
          icon={<Icon icon={XIcon} size="sm" />}
          onClick={() => {
            onChange(withoutField(filters, 'quote'));
          }}
        />
      )}
    </HStack>
  );
}

function UpcomingChip({
  filters,
  onChange,
}: {
  filters: PowerSearchFilter[];
  onChange: (next: PowerSearchFilter[]) => void;
}) {
  const isSet = isUpcomingActive(filters);
  return (
    <HStack gap={0} vAlign="center">
      <Button
        label={isSet ? 'Upcoming is scheduled from now on' : 'Upcoming'}
        variant={isSet ? 'secondary' : 'ghost'}
        size="md"
        onClick={() => {
          onChange(toggleUpcoming(filters));
        }}
      />
      {isSet && (
        <IconButton
          label="Clear Upcoming filter"
          variant="ghost"
          size="sm"
          icon={<Icon icon={XIcon} size="sm" />}
          onClick={() => {
            onChange(toggleUpcoming(filters));
          }}
        />
      )}
    </HStack>
  );
}

export function FilterBar({
  filters,
  onFiltersChange,
  resultCount,
  onOpenViewOptions,
}: {
  filters: PowerSearchFilter[];
  onFiltersChange: (next: PowerSearchFilter[]) => void;
  resultCount: number;
  onOpenViewOptions: () => void;
}) {
  const [isAdvanced, setIsAdvanced] = useState(false);
  const resultLabel = `${resultCount} result${resultCount === 1 ? '' : 's'}`;

  if (isAdvanced) {
    return (
      <HStack gap={2} vAlign="center">
        <StackItem size="fill">
          <PowerSearch
            config={POWER_SEARCH_CONFIG}
            filters={filters}
            onChange={(next) => {
              onFiltersChange([...next]);
            }}
            placeholder="Search jobs with field:operator:value tokens..."
            resultCount={resultLabel}
          />
        </StackItem>
        <Button
          label="Simple filters"
          variant="secondary"
          size="md"
          icon={<Icon icon={ListMagnifyingGlassIcon} size="sm" />}
          onClick={() => {
            setIsAdvanced(false);
          }}
        />
        <Button label="View options" variant="secondary" size="md" onClick={onOpenViewOptions} />
      </HStack>
    );
  }

  return (
    <HStack gap={2} vAlign="center" wrap="wrap">
      <TextInput
        label="Search jobs"
        isLabelHidden
        placeholder="Search by job name"
        startIcon={<Icon icon={MagnifyingGlassIcon} size="sm" />}
        hasClear
        value={searchText(filters)}
        onChange={(value) => {
          onFiltersChange(setSearchText(filters, value));
        }}
        width={220}
      />
      <UpcomingChip filters={filters} onChange={onFiltersChange} />
      <EnumFilterChip
        label="Status"
        field="status"
        options={JOB_STATUSES.map((value) => ({
          value,
          label: STATUS_LABEL[value],
        }))}
        filters={filters}
        onChange={onFiltersChange}
      />
      <EnumFilterChip
        label="Customer"
        field="customer"
        options={CUSTOMERS.map((c) => ({ value: c.id, label: c.name }))}
        filters={filters}
        onChange={onFiltersChange}
      />
      <EnumFilterChip
        label="Technician"
        field="technician"
        options={TECHNICIANS.map((t) => ({ value: t.id, label: t.name }))}
        filters={filters}
        onChange={onFiltersChange}
      />
      <EnumFilterChip
        label="Priority"
        field="priority"
        options={JOB_PRIORITIES.map((value) => ({
          value,
          label: PRIORITY_LABEL[value],
        }))}
        filters={filters}
        onChange={onFiltersChange}
      />
      <QuoteFilterChip filters={filters} onChange={onFiltersChange} />

      <IconButton
        label="Switch to advanced search"
        tooltip="Advanced search"
        variant="ghost"
        size="md"
        icon={<Icon icon={FunnelIcon} size="sm" />}
        onClick={() => {
          setIsAdvanced(true);
        }}
      />

      <StackItem size="fill" />
      <HStack gap={3} vAlign="center">
        <Text type="supporting" color="secondary">
          {resultLabel}
        </Text>
        <Button
          label="View options"
          variant="secondary"
          size="md"
          icon={<Icon icon={SlidersHorizontalIcon} size="sm" />}
          onClick={onOpenViewOptions}
        />
      </HStack>
    </HStack>
  );
}
