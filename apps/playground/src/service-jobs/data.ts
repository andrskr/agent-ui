import type {
  Customer,
  Job,
  JobPriority,
  JobStatus,
  Technician,
} from '#/service-jobs/service-jobs-types.ts';

export const CUSTOMERS: Customer[] = [
  { id: 'cust-riverside', name: 'Riverside Medical Center' },
  { id: 'cust-northgate', name: 'Northgate Logistics' },
  { id: 'cust-summit', name: 'Summit Ridge Apartments' },
  { id: 'cust-harborview', name: 'Harborview School District' },
  { id: 'cust-lakeside', name: 'Lakeside Manufacturing' },
  { id: 'cust-parkview', name: 'Parkview Hotel Group' },
];

export const TECHNICIANS: Technician[] = [
  { id: 'tech-nguyen', name: 'Priya Nguyen' },
  { id: 'tech-osei', name: 'Kwame Osei' },
  { id: 'tech-castillo', name: 'Marisol Castillo' },
  { id: 'tech-berg', name: 'Erik Berg' },
  { id: 'tech-farah', name: 'Amina Farah' },
];

const JOB_TITLES = [
  'Rooftop HVAC unit inspection',
  'Chiller compressor replacement',
  'Boiler pressure relief valve service',
  'Air handler belt replacement',
  'Refrigerant leak diagnosis',
  'Ductwork cleaning and balancing',
  'Cooling tower descale',
  'Thermostat and controls upgrade',
  'Emergency generator tie-in',
  'Water heater flush and inspection',
  'Exhaust fan motor replacement',
  'Ventilation damper actuator repair',
  'Split system installation',
  'Condenser coil cleaning',
  'Building automation system tuning',
  'Gas furnace ignition repair',
  'Kitchen hood suppression test',
  'Chilled water loop repair',
  'Rooftop curb and unit resealing',
  'Variable frequency drive replacement',
];

const STATUS_CYCLE: JobStatus[] = [
  'overdue',
  'scheduled',
  'in_progress',
  'scheduled',
  'on_hold',
  'completed',
  'scheduled',
  'in_progress',
  'completed',
  'overdue',
];

const PRIORITY_CYCLE: JobPriority[] = ['normal', 'normal', 'high', 'urgent', 'normal', 'high'];

// Fixed-seed PRNG (mulberry32) so sample data is stable across runs without
// relying on hand-writing hundreds of rows.
function createRng(seed: number) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d_2b_79_f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t = (t + Math.imul(t ^ (t >>> 7), t | 61)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4_294_967_296;
  };
}

const TOTAL_JOBS = 165;
const SCHEDULE_START = Date.UTC(2026, 7, 24, 7, 0); // Mon Aug 24 2026, 07:00 UTC
const SCHEDULE_STEP_MINUTES = 95;

function buildJobs(): Job[] {
  const rng = createRng(20_260_824);
  const jobs: Job[] = [];
  for (let i = 0; i < TOTAL_JOBS; i++) {
    const customer = CUSTOMERS[i % CUSTOMERS.length];
    const technician = TECHNICIANS[(i * 3 + 1) % TECHNICIANS.length];
    const title = JOB_TITLES[Math.floor(rng() * JOB_TITLES.length)];
    const status = STATUS_CYCLE[i % STATUS_CYCLE.length];
    const priority = PRIORITY_CYCLE[i % PRIORITY_CYCLE.length];
    const scheduledAt = new Date(SCHEDULE_START + i * SCHEDULE_STEP_MINUTES * 60_000).toISOString();
    const quote = Math.round((250 + rng() * 5750) / 5) * 5;
    jobs.push({
      id: `job-${i + 1}`,
      jobNumber: `SJ-${(10_240 + i).toString()}`,
      title,
      customerId: customer.id,
      technicianId: technician.id,
      status,
      priority,
      scheduledAt,
      quote,
    });
  }
  return jobs;
}

export const JOBS: Job[] = buildJobs();

export function customerName(customerId: string): string {
  return CUSTOMERS.find((c) => c.id === customerId)?.name ?? customerId;
}

export function technicianName(technicianId: string): string {
  return TECHNICIANS.find((t) => t.id === technicianId)?.name ?? technicianId;
}
