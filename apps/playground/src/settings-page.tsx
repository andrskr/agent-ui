import { Badge } from '@astryxdesign/core/Badge';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { Center } from '@astryxdesign/core/Center';
import { Divider } from '@astryxdesign/core/Divider';
import { useMediaQuery } from '@astryxdesign/core/hooks';
import { Icon } from '@astryxdesign/core/Icon';
import { HStack, Layout, LayoutContent, LayoutPanel, VStack } from '@astryxdesign/core/Layout';
import { Link } from '@astryxdesign/core/Link';
import { List, ListItem } from '@astryxdesign/core/List';
import { Selector } from '@astryxdesign/core/Selector';
import { StatusDot } from '@astryxdesign/core/StatusDot';
import { Switch } from '@astryxdesign/core/Switch';
import { Heading, Text } from '@astryxdesign/core/Text';
import { TextInput } from '@astryxdesign/core/TextInput';
import { colorVars, radiusVars, spacingVars } from '@astryxdesign/core/theme/tokens.stylex';
import { Toolbar } from '@astryxdesign/core/Toolbar';
import {
  ArrowLeftIcon,
  BellIcon,
  BriefcaseIcon,
  CaretRightIcon,
  CreditCardIcon,
  FileTextIcon,
  GlobeIcon,
  LockIcon,
  PencilSimpleIcon,
  QuestionIcon,
  ShareNetworkIcon,
  ShieldCheckIcon,
  UserIcon,
} from '@phosphor-icons/react';
import * as stylex from '@stylexjs/stylex';
import type { ComponentType } from 'react';
import { useState } from 'react';

import type {
  Account,
  IdentityVerificationStatus,
  PersonalInformation,
  PrivacySettings,
  RegionPreferences,
  SelectOption,
} from '#/account-data.ts';
import { sampleAccount } from '#/account-data.ts';

const styles = stylex.create({
  page: { minHeight: '100dvh' },
  contentSurface: { backgroundColor: colorVars['--color-background-surface'] },
  navPadding: {
    paddingBlock: spacingVars['--spacing-4'],
    paddingInline: spacingVars['--spacing-3'],
  },
  navHeading: { marginInline: spacingVars['--spacing-4'] },
  rowPadding: { paddingBlock: spacingVars['--spacing-4'] },
  iconBox: {
    borderRadius: radiusVars['--radius-container'],
    backgroundColor: colorVars['--color-background-surface'],
    flexShrink: 0,
  },
});

// The nav lists every section. Only BUILT_SECTIONS have panels for this first
// pass; the rest exist in the nav but show a "not available yet" placeholder.
interface NavItem {
  label: string;
  icon: ComponentType;
}

const NAV_ITEMS: NavItem[] = [
  { label: 'Personal information', icon: UserIcon },
  { label: 'Login & security', icon: LockIcon },
  { label: 'Privacy', icon: ShieldCheckIcon },
  { label: 'Notifications', icon: BellIcon },
  { label: 'Taxes', icon: FileTextIcon },
  { label: 'Payments', icon: CreditCardIcon },
  { label: 'Languages & currency', icon: GlobeIcon },
  { label: 'Travel for work', icon: BriefcaseIcon },
];

const BUILT_SECTIONS = new Set(['Personal information', 'Privacy', 'Languages & currency']);

// Shorter titles shown beside the mobile back button; falls back to the nav label.
const MOBILE_TITLES = new Map<string, string>([
  ['Personal information', 'Personal info'],
  ['Privacy', 'Privacy'],
  ['Languages & currency', 'Languages & currency'],
]);

const VERIFICATION_DISPLAY = {
  verified: { variant: 'success', label: 'Verified' },
  pending: { variant: 'warning', label: 'Pending review' },
  unverified: { variant: 'neutral', label: 'Not verified' },
} satisfies Record<
  IdentityVerificationStatus,
  { variant: 'success' | 'warning' | 'neutral'; label: string }
>;

const NOT_PROVIDED = 'Not provided';

function optionLabel(options: SelectOption[], value: string): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

// A read-only row: label on the left, value or a right-aligned action on the
// right. Used for identity verification and the blocked-people link.
function InfoRow({
  label,
  value,
  action,
}: {
  label: string;
  value?: string;
  action?: React.ReactNode;
}) {
  return (
    <>
      <HStack hAlign="between" vAlign="center" xstyle={styles.rowPadding}>
        <VStack gap={0}>
          <Text type="body" weight="semibold" display="block">
            {label}
          </Text>
          {value !== undefined && (
            <Text type="supporting" color="secondary" display="block">
              {value}
            </Text>
          )}
        </VStack>
        {action}
      </HStack>
      <Divider />
    </>
  );
}

interface EditableRowProps {
  label: string;
  value: string;
  isExpanded: boolean;
  onEdit: () => void;
  onCancel: () => void;
  onSave: () => void;
  children: React.ReactNode;
}

// A row that expands in place to reveal an editor with Save / Cancel.
function EditableRow({
  label,
  value,
  isExpanded,
  onEdit,
  onCancel,
  onSave,
  children,
}: EditableRowProps) {
  return (
    <>
      {isExpanded ? (
        <VStack gap={4} xstyle={styles.rowPadding}>
          <Text type="body" weight="semibold" display="block">
            {label}
          </Text>
          {children}
          <HStack gap={2}>
            <Button label="Save" variant="primary" onClick={onSave} />
            <Button label="Cancel" variant="ghost" onClick={onCancel} />
          </HStack>
        </VStack>
      ) : (
        <HStack hAlign="between" vAlign="start" xstyle={styles.rowPadding}>
          <VStack gap={0}>
            <Text type="body" weight="semibold" display="block">
              {label}
            </Text>
            <Text type="supporting" color="secondary" display="block">
              {value || NOT_PROVIDED}
            </Text>
          </VStack>
          <Link
            href="#"
            onClick={(event: React.MouseEvent) => {
              event.preventDefault();
              onEdit();
            }}
          >
            Edit
          </Link>
        </HStack>
      )}
      <Divider />
    </>
  );
}

function SectionHeading({ title, isNarrow }: { title: string; isNarrow: boolean }) {
  if (isNarrow) return null;
  return <Heading level={2}>{title}</Heading>;
}

export default function SettingsPage({ account = sampleAccount }: { account?: Account }) {
  const isNarrow = useMediaQuery('(max-width: 768px)');
  // Mobile is a master -> detail drill-down: 'nav' shows the menu, 'detail'
  // shows the selected section with a back button. Desktop shows both.
  const [mobileView, setMobileView] = useState<'nav' | 'detail'>('nav');
  const [activeNav, setActiveNav] = useState('Personal information');
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  const [personal, setPersonal] = useState<PersonalInformation>(account.personalInformation);
  const [privacy, setPrivacy] = useState<PrivacySettings>(account.privacy);
  const [preferences, setPreferences] = useState<RegionPreferences>(account.preferences);
  const { languages, currencies, timeZones } = account.supportedOptions;

  function selectNav(label: string) {
    setActiveNav(label);
    setExpandedRow(null);
    setMobileView('detail');
  }

  function setPersonalField(key: keyof PersonalInformation) {
    return (value: string) => {
      setPersonal((current) => ({ ...current, [key]: value }));
    };
  }

  function collapse() {
    setExpandedRow(null);
  }

  const navList = (
    <VStack gap={4} xstyle={styles.navPadding}>
      <Heading level={2} xstyle={styles.navHeading}>
        Account settings
      </Heading>
      <List density="spacious">
        {NAV_ITEMS.map((item) => (
          <ListItem
            key={item.label}
            label={item.label}
            startContent={<Icon icon={item.icon} />}
            endContent={
              isNarrow ? <Icon icon={CaretRightIcon} size="sm" color="secondary" /> : undefined
            }
            isSelected={!isNarrow && activeNav === item.label}
            onClick={() => {
              selectNav(item.label);
            }}
          />
        ))}
      </List>
    </VStack>
  );

  // Mobile, nav view: show only the menu.
  if (isNarrow && mobileView === 'nav') {
    return (
      <Layout
        height="fill"
        xstyle={styles.page}
        content={
          <LayoutContent padding={2} xstyle={styles.contentSurface}>
            {navList}
          </LayoutContent>
        }
      />
    );
  }

  const verification = VERIFICATION_DISPLAY[personal.identityVerification];

  return (
    <Layout
      height="fill"
      contentWidth={1200}
      xstyle={styles.page}
      start={
        isNarrow ? undefined : (
          <LayoutPanel hasDivider padding={0}>
            {navList}
          </LayoutPanel>
        )
      }
      content={
        <LayoutContent padding={4} xstyle={styles.contentSurface}>
          <VStack gap={0}>
            {isNarrow && (
              <Toolbar
                label={`Back to Account settings — ${MOBILE_TITLES.get(activeNav) ?? activeNav}`}
                gap={2}
                startContent={
                  <>
                    <Button
                      label="Back to Account settings"
                      variant="ghost"
                      size="sm"
                      isIconOnly
                      icon={<Icon icon={ArrowLeftIcon} size="sm" />}
                      onClick={() => {
                        setMobileView('nav');
                      }}
                    />
                    <Heading level={2}>{MOBILE_TITLES.get(activeNav) ?? activeNav}</Heading>
                  </>
                }
              />
            )}

            {activeNav === 'Personal information' && (
              <VStack gap={6}>
                <SectionHeading title="Personal info" isNarrow={isNarrow} />
                <VStack gap={0}>
                  <EditableRow
                    label="Legal name"
                    value={personal.legalName}
                    isExpanded={expandedRow === 'legalName'}
                    onEdit={() => {
                      setExpandedRow('legalName');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Legal name"
                      isLabelHidden
                      value={personal.legalName}
                      onChange={setPersonalField('legalName')}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Preferred first name"
                    value={personal.preferredFirstName}
                    isExpanded={expandedRow === 'preferredFirstName'}
                    onEdit={() => {
                      setExpandedRow('preferredFirstName');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Preferred first name"
                      isLabelHidden
                      value={personal.preferredFirstName}
                      onChange={setPersonalField('preferredFirstName')}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Email address"
                    value={personal.email}
                    isExpanded={expandedRow === 'email'}
                    onEdit={() => {
                      setExpandedRow('email');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Email address"
                      isLabelHidden
                      value={personal.email}
                      onChange={setPersonalField('email')}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Phone number"
                    value={personal.phone}
                    isExpanded={expandedRow === 'phone'}
                    onEdit={() => {
                      setExpandedRow('phone');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Phone number"
                      isLabelHidden
                      value={personal.phone}
                      onChange={setPersonalField('phone')}
                    />
                  </EditableRow>
                  {/* Identity verification is read-only: it comes from the KYC partner. */}
                  <InfoRow
                    label="Identity verification"
                    action={
                      <HStack gap={2} vAlign="center">
                        <StatusDot variant={verification.variant} label={verification.label} />
                        <Text type="body" color="secondary">
                          {verification.label}
                        </Text>
                      </HStack>
                    }
                  />
                  <EditableRow
                    label="Residential address"
                    value={personal.residentialAddress}
                    isExpanded={expandedRow === 'residentialAddress'}
                    onEdit={() => {
                      setExpandedRow('residentialAddress');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Residential address"
                      isLabelHidden
                      value={personal.residentialAddress}
                      onChange={setPersonalField('residentialAddress')}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Mailing address"
                    value={personal.mailingAddress}
                    isExpanded={expandedRow === 'mailingAddress'}
                    onEdit={() => {
                      setExpandedRow('mailingAddress');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Mailing address"
                      isLabelHidden
                      value={personal.mailingAddress}
                      onChange={setPersonalField('mailingAddress')}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Emergency contact"
                    value={personal.emergencyContact}
                    isExpanded={expandedRow === 'emergencyContact'}
                    onEdit={() => {
                      setExpandedRow('emergencyContact');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <TextInput
                      label="Emergency contact"
                      isLabelHidden
                      value={personal.emergencyContact}
                      onChange={setPersonalField('emergencyContact')}
                    />
                  </EditableRow>
                </VStack>

                <Card padding={4}>
                  <VStack gap={4}>
                    <HStack gap={3} vAlign="start">
                      <Center width={48} height={48} xstyle={styles.iconBox}>
                        <Icon icon={LockIcon} />
                      </Center>
                      <VStack gap={0}>
                        <Text type="body" weight="semibold" display="block">
                          Why isn&apos;t my info shown here?
                        </Text>
                        <Text type="supporting" color="secondary" display="block">
                          We hide some account details to protect your identity.
                        </Text>
                      </VStack>
                    </HStack>
                    <Divider />
                    <HStack gap={3} vAlign="start">
                      <Center width={48} height={48} xstyle={styles.iconBox}>
                        <Icon icon={PencilSimpleIcon} />
                      </Center>
                      <VStack gap={0}>
                        <Text type="body" weight="semibold" display="block">
                          Which details can be edited?
                        </Text>
                        <Text type="supporting" color="secondary" display="block">
                          You can edit your contact info and personal details. Identity verification
                          is managed by our KYC partner and cannot be changed here.
                        </Text>
                      </VStack>
                    </HStack>
                    <Divider />
                    <HStack gap={3} vAlign="start">
                      <Center width={48} height={48} xstyle={styles.iconBox}>
                        <Icon icon={ShareNetworkIcon} />
                      </Center>
                      <VStack gap={0}>
                        <Text type="body" weight="semibold" display="block">
                          What info is shared with others?
                        </Text>
                        <Text type="supporting" color="secondary" display="block">
                          We only release contact information after a reservation is confirmed.
                        </Text>
                      </VStack>
                    </HStack>
                  </VStack>
                </Card>
              </VStack>
            )}

            {activeNav === 'Privacy' && (
              <VStack gap={6}>
                <SectionHeading title="Privacy" isNarrow={isNarrow} />
                <VStack gap={8}>
                  <VStack gap={0}>
                    <Heading level={3}>Messages</Heading>
                    <VStack xstyle={styles.rowPadding}>
                      <Switch
                        label="Show people when I have read their messages"
                        size="sm"
                        value={privacy.readReceipts}
                        onChange={(value) => {
                          setPrivacy((current) => ({ ...current, readReceipts: value }));
                        }}
                        labelPosition="start"
                        labelSpacing="spread"
                      />
                    </VStack>
                    <InfoRow label="Blocked people" action={<Link href="#">View</Link>} />
                  </VStack>

                  <VStack gap={0}>
                    <Heading level={3}>Listings</Heading>
                    <VStack xstyle={styles.rowPadding}>
                      <Switch
                        label="Include my listings in search engines"
                        size="sm"
                        description="Turning this on means search engines, like Google, will show your listing pages in search results. Anyone can then find them."
                        value={privacy.searchEngineIndexing}
                        onChange={(value) => {
                          setPrivacy((current) => ({ ...current, searchEngineIndexing: value }));
                        }}
                        labelPosition="start"
                        labelSpacing="spread"
                      />
                    </VStack>
                    <Divider />
                  </VStack>

                  <VStack gap={4}>
                    <Heading level={3}>Reviews</Heading>
                    <Text type="supporting" color="secondary">
                      Choose what is shared when you write a review.
                    </Text>
                    <Switch
                      label="Show my home city and country"
                      size="sm"
                      value={privacy.reviewSharing.homeCity}
                      onChange={(value) => {
                        setPrivacy((current) => ({
                          ...current,
                          reviewSharing: { ...current.reviewSharing, homeCity: value },
                        }));
                      }}
                      labelPosition="start"
                      labelSpacing="spread"
                    />
                    <Switch
                      label="Show my trip type"
                      size="sm"
                      description="Ex: stayed with kids or pets"
                      value={privacy.reviewSharing.tripType}
                      onChange={(value) => {
                        setPrivacy((current) => ({
                          ...current,
                          reviewSharing: { ...current.reviewSharing, tripType: value },
                        }));
                      }}
                      labelPosition="start"
                      labelSpacing="spread"
                    />
                    <Switch
                      label="Show my length of stay"
                      size="sm"
                      description="Ex: a few nights, about a week"
                      value={privacy.reviewSharing.lengthOfStay}
                      onChange={(value) => {
                        setPrivacy((current) => ({
                          ...current,
                          reviewSharing: { ...current.reviewSharing, lengthOfStay: value },
                        }));
                      }}
                      labelPosition="start"
                      labelSpacing="spread"
                    />
                  </VStack>

                  <VStack gap={4}>
                    <Heading level={3}>Data privacy</Heading>
                    <Switch
                      label="Help improve AI-powered features"
                      size="sm"
                      description="When this is on, we use your data to develop and improve AI models."
                      value={privacy.aiFeatures}
                      onChange={(value) => {
                        setPrivacy((current) => ({ ...current, aiFeatures: value }));
                      }}
                      labelPosition="start"
                      labelSpacing="spread"
                    />
                    <Card variant="muted">
                      <HStack gap={4} vAlign="start">
                        <Center width={48} height={48} xstyle={styles.iconBox}>
                          <Icon icon={ShieldCheckIcon} />
                        </Center>
                        <VStack gap={1}>
                          <Text type="body" weight="bold">
                            Committed to privacy
                          </Text>
                          <Text type="supporting" color="secondary">
                            We are committed to keeping your data protected. See details in our{' '}
                            <Link href="#" type="supporting">
                              Privacy Policy
                            </Link>
                            .
                          </Text>
                        </VStack>
                      </HStack>
                    </Card>
                  </VStack>
                </VStack>
              </VStack>
            )}

            {activeNav === 'Languages & currency' && (
              <VStack gap={6}>
                <SectionHeading title="Languages & currency" isNarrow={isNarrow} />
                <VStack gap={0}>
                  <EditableRow
                    label="Preferred language"
                    value={optionLabel(languages, preferences.language)}
                    isExpanded={expandedRow === 'language'}
                    onEdit={() => {
                      setExpandedRow('language');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <Selector
                      label="Language"
                      isLabelHidden
                      size="lg"
                      value={preferences.language}
                      onChange={(value) => {
                        setPreferences((current) => ({ ...current, language: value }));
                      }}
                      options={languages}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Preferred currency"
                    value={optionLabel(currencies, preferences.currency)}
                    isExpanded={expandedRow === 'currency'}
                    onEdit={() => {
                      setExpandedRow('currency');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <Selector
                      label="Currency"
                      isLabelHidden
                      size="lg"
                      value={preferences.currency}
                      onChange={(value) => {
                        setPreferences((current) => ({ ...current, currency: value }));
                      }}
                      options={currencies}
                    />
                  </EditableRow>
                  <EditableRow
                    label="Time zone"
                    value={optionLabel(timeZones, preferences.timeZone)}
                    isExpanded={expandedRow === 'timeZone'}
                    onEdit={() => {
                      setExpandedRow('timeZone');
                    }}
                    onCancel={collapse}
                    onSave={collapse}
                  >
                    <Selector
                      label="Time zone"
                      isLabelHidden
                      size="lg"
                      value={preferences.timeZone}
                      onChange={(value) => {
                        setPreferences((current) => ({ ...current, timeZone: value }));
                      }}
                      options={timeZones}
                    />
                  </EditableRow>
                </VStack>
              </VStack>
            )}

            {!BUILT_SECTIONS.has(activeNav) && (
              <VStack gap={6}>
                <SectionHeading title={activeNav} isNarrow={isNarrow} />
                <Card variant="muted">
                  <HStack gap={4} vAlign="center">
                    <Center width={48} height={48} xstyle={styles.iconBox}>
                      <Icon icon={QuestionIcon} />
                    </Center>
                    <VStack gap={1}>
                      <HStack gap={2} vAlign="center">
                        <Text type="body" weight="bold">
                          {activeNav}
                        </Text>
                        <Badge label="Coming soon" />
                      </HStack>
                      <Text type="supporting" color="secondary">
                        This section is not available yet.
                      </Text>
                    </VStack>
                  </HStack>
                </Card>
              </VStack>
            )}
          </VStack>
        </LayoutContent>
      }
    />
  );
}
