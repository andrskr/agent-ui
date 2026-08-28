// Sample account data for the settings screen. The real values come from the
// API later; the page takes an `Account` as a prop and matches this shape.

export type IdentityVerificationStatus = 'verified' | 'pending' | 'unverified';

export interface SelectOption {
  label: string;
  value: string;
}

export interface PersonalInformation {
  legalName: string;
  preferredFirstName: string;
  email: string;
  phone: string;
  // Comes from the KYC partner. The user cannot change it here.
  identityVerification: IdentityVerificationStatus;
  residentialAddress: string;
  mailingAddress: string;
  emergencyContact: string;
}

export interface ReviewSharing {
  homeCity: boolean;
  tripType: boolean;
  lengthOfStay: boolean;
}

export interface PrivacySettings {
  readReceipts: boolean;
  searchEngineIndexing: boolean;
  reviewSharing: ReviewSharing;
  aiFeatures: boolean;
}

export interface RegionPreferences {
  // Each value matches an option in `supportedOptions`.
  language: string;
  currency: string;
  timeZone: string;
}

export interface SupportedOptions {
  languages: SelectOption[];
  currencies: SelectOption[];
  timeZones: SelectOption[];
}

export interface Account {
  personalInformation: PersonalInformation;
  privacy: PrivacySettings;
  preferences: RegionPreferences;
  supportedOptions: SupportedOptions;
}

const supportedOptions: SupportedOptions = {
  languages: [
    { label: 'English (United States)', value: 'en-US' },
    { label: 'English (United Kingdom)', value: 'en-GB' },
    { label: 'Français', value: 'fr-FR' },
    { label: 'Español', value: 'es-ES' },
    { label: 'Deutsch', value: 'de-DE' },
    { label: 'Português (Brasil)', value: 'pt-BR' },
    { label: '日本語', value: 'ja-JP' },
  ],
  currencies: [
    { label: 'US dollar (USD)', value: 'USD' },
    { label: 'Euro (EUR)', value: 'EUR' },
    { label: 'British pound (GBP)', value: 'GBP' },
    { label: 'Canadian dollar (CAD)', value: 'CAD' },
    { label: 'Australian dollar (AUD)', value: 'AUD' },
    { label: 'Japanese yen (JPY)', value: 'JPY' },
  ],
  timeZones: [
    { label: '(GMT-08:00) Pacific Time', value: 'America/Los_Angeles' },
    { label: '(GMT-05:00) Eastern Time', value: 'America/New_York' },
    { label: '(GMT+00:00) London', value: 'Europe/London' },
    { label: '(GMT+01:00) Paris', value: 'Europe/Paris' },
    { label: '(GMT+09:00) Tokyo', value: 'Asia/Tokyo' },
    { label: '(GMT+00:00) UTC', value: 'UTC' },
  ],
};

// Realistic values, with some fields left empty on purpose so the empty states
// are visible: preferred first name and both addresses are not filled in.
export const sampleAccount: Account = {
  personalInformation: {
    legalName: 'Maya Okonkwo',
    preferredFirstName: '',
    email: 'maya.okonkwo@example.com',
    phone: '+1 (415) 555-0142',
    identityVerification: 'verified',
    residentialAddress: '',
    mailingAddress: '',
    emergencyContact: 'Daniel Okonkwo · +1 (415) 555-0177',
  },
  privacy: {
    readReceipts: true,
    searchEngineIndexing: false,
    reviewSharing: {
      homeCity: true,
      tripType: true,
      lengthOfStay: false,
    },
    aiFeatures: false,
  },
  preferences: {
    language: 'en-US',
    currency: 'USD',
    timeZone: 'America/Los_Angeles',
  },
  supportedOptions,
};
