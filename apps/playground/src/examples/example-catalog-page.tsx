import { Card } from '@astryxdesign/core/Card';
import { ClickableCard } from '@astryxdesign/core/ClickableCard';
import { Icon } from '@astryxdesign/core/Icon';
import { HStack, Layout, LayoutContent, VStack } from '@astryxdesign/core/Layout';
import { Heading, Text } from '@astryxdesign/core/Text';
import { ArrowRightIcon, GearSixIcon, WrenchIcon } from '@phosphor-icons/react';

const examples = [
  {
    title: 'Settings',
    description: 'Account settings with profile, privacy, and regional preferences.',
    href: '/settings',
    icon: GearSixIcon,
  },
  {
    title: 'Service jobs',
    description: 'Filterable, groupable job table for a field-service scheduling team.',
    href: '/service-jobs',
    icon: WrenchIcon,
  },
];

export function ExampleCatalogPage() {
  return (
    <Layout
      height="fill"
      contentWidth={720}
      content={
        <LayoutContent padding={8}>
          <VStack gap={8}>
            <VStack gap={3}>
              <Text type="label" color="accent">
                Something Something UI
              </Text>
              <VStack gap={2}>
                <Heading level={1}>Interface examples</Heading>
                <Text type="large" color="secondary">
                  A collection of real product surfaces built with Astryx. Pick one to explore.
                </Text>
              </VStack>
            </VStack>

            <VStack gap={3}>
              <Heading level={2}>Pages</Heading>
              <VStack gap={2}>
                {examples.map((example) => (
                  <ClickableCard
                    key={example.href}
                    href={example.href}
                    label={example.title}
                    padding={4}
                    elevation="low"
                  >
                    <HStack justify="between" vAlign="center" gap={4}>
                      <HStack gap={4} vAlign="center">
                        <Card variant="muted" padding={2}>
                          <Icon icon={example.icon} size="md" color="accent" />
                        </Card>
                        <VStack gap={1}>
                          <Heading level={3}>{example.title}</Heading>
                          <Text color="secondary" size="sm">
                            {example.description}
                          </Text>
                        </VStack>
                      </HStack>
                      <Icon icon={ArrowRightIcon} size="sm" color="secondary" />
                    </HStack>
                  </ClickableCard>
                ))}
              </VStack>
            </VStack>
          </VStack>
        </LayoutContent>
      }
    />
  );
}
