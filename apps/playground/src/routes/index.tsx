import { Text } from '@astryxdesign/core/Text';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/')({
  component: HomePage,
});

function HomePage() {
  return <Text>Hello</Text>;
}
