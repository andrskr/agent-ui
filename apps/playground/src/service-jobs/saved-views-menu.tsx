import { Button } from '@astryxdesign/core/Button';
import { Dialog, DialogHeader } from '@astryxdesign/core/Dialog';
import { DropdownMenu } from '@astryxdesign/core/DropdownMenu';
import { HStack, Layout, LayoutContent, LayoutFooter } from '@astryxdesign/core/Layout';
import { TextInput } from '@astryxdesign/core/TextInput';
import { FloppyDiskIcon } from '@phosphor-icons/react';
import { useState } from 'react';

import type { SavedView } from '#/service-jobs/service-jobs-types.ts';

export function SavedViewsMenu({
  savedViews,
  activeViewName,
  onApplyView,
  onSaveView,
}: {
  savedViews: SavedView[];
  activeViewName: string | null;
  onApplyView: (view: SavedView) => void;
  onSaveView: (name: string) => void;
}) {
  const [isSaveDialogOpen, setIsSaveDialogOpen] = useState(false);
  const [name, setName] = useState('');

  return (
    <>
      <DropdownMenu
        button={{ label: activeViewName ?? 'Views', variant: 'secondary', size: 'md' }}
        items={[
          {
            type: 'section',
            title: 'Saved views',
            items: savedViews.map((view) => ({
              id: view.id,
              label: view.name,
              onClick: () => {
                onApplyView(view);
              },
            })),
          },
          { type: 'divider' },
          {
            label: 'Save current view...',
            icon: FloppyDiskIcon,
            onClick: () => {
              setIsSaveDialogOpen(true);
            },
          },
        ]}
      />
      <Dialog
        isOpen={isSaveDialogOpen}
        onOpenChange={setIsSaveDialogOpen}
        purpose="form"
        width={400}
      >
        <Layout
          header={<DialogHeader title="Save view" onOpenChange={setIsSaveDialogOpen} />}
          content={
            <LayoutContent padding={4}>
              <TextInput
                label="View name"
                value={name}
                onChange={setName}
                isRequired
                placeholder="e.g. My open jobs"
              />
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
                    setIsSaveDialogOpen(false);
                  }}
                />
                <Button
                  label="Save view"
                  variant="primary"
                  size="md"
                  isDisabled={!name.trim()}
                  onClick={() => {
                    onSaveView(name.trim());
                    setName('');
                    setIsSaveDialogOpen(false);
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
