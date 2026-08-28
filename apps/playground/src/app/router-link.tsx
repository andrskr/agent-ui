import { Link } from '@tanstack/react-router';
import type { ComponentProps } from 'react';

export function RouterLink({ href, ...props }: ComponentProps<'a'>) {
  if (href === undefined) {
    throw new Error('Astryx links rendered by RouterLink require an href');
  }

  return <Link {...props} to={href} />;
}
