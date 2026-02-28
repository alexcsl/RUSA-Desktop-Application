// routing.ts — Role-to-route mapping
// Source of truth: AUTH_GUIDE.md §7
//
// After a successful login, Svelte reads currentUser.role and routes to
// the appropriate landing page. This is the only place role-based routing
// is driven by the frontend alone — all subsequent protections use RouteGuard.

export function getDefaultRoute(role: string): string {
  const routeMap: Record<string, string> = {
    AgriculturalEngineer: '/engineers/experiments',
    BiologicalEngineer: '/engineers/experiments',
    DataAnalyst: '/data/analyst/inbox',
    GalacticSecurityHead: '/security/incidents',
    GalacticSecurityStaff: '/security/tasks',
    Mathematician: '/scientists/tasks',
    Physicist: '/scientists/experiments',
    Chemist: '/scientists/experiments',
    Biologist: '/scientists/experiments',
    Astronaut: '/astronauts/missions',
    SettlerCommander: '/settlers/tasks',
    CivilEngineer: '/settlers/tasks',
    Farmer: '/settlers/tasks',
    TemporarySetter: '/settlers/tasks',
    SpaceStationSettler: '/station/overview',
    Psychiatrist: '/psychiatry/patients',
    PsychiatristAssistant: '/psychiatry/assistant/patients',
    MedicalStaff: '/medical/treatment',
    HeadOfMedicine: '/medical/head/shifts',
    HeadOfSanitary: '/sanitary/head/scheduling',
    InspectorCrew: '/sanitary/tasks',
    DisposalCrew: '/sanitary/disposal/docs',
    WastewaterCrew: '/sanitary/wastewater/docs',
    CleanupCrew: '/sanitary/tasks',
    TransportCrew: '/sanitary/tasks',
    // All Director roles → directors dashboard
    GeneralDirector: '/directors/votes',
    TheDirector: '/directors/votes',
    TheAccountant: '/directors/accountant/queue',
    TheLibrarian: '/directors/librarian/archive',
    TheNomad: '/directors/nomad/relocate',
    TheArtificer: '/directors/artificer/tasks/new',
    TheObserver: '/directors/observer/tasks/new',
    TheWanderer: '/directors/wanderer/missions/new',
    TheTaskmaster: '/directors/taskmaster/dashboard',
    TheGuardian: '/directors/guardian/security-reports',
    TheStatistician: '/directors/statistician/requests',
    TheCoordinator: '/directors/coordinator/events/new',
    TheOverseer: '/directors/overseer/security-line',
    TheAnchorman: '/directors/anchorman/broadcast-requests',
    Administrator: '/admin/votes',
  };
  return routeMap[role] ?? '/dashboard';
}

// ─── Messaging route helpers ─────────────────────────────────────────
// Returns the best messaging-entry route for a given role.
// Used by navigation components to provide a "Messages" link.

export function getMessagingRoute(role: string): string | null {
  // Dedicated channel pages for specific roles
  const securityRoles = [
    'GalacticSecurityHead', 'GalacticSecurityStaff',
    'TheGuardian', 'TheOverseer',
  ];
  if (securityRoles.includes(role)) return '/messaging/channels/security';
  if (role === 'HeadOfMedicine') return '/messaging/channels/medical-heads';
  // Administrator sees all channels via generic inbox
  if (role === 'Administrator') return '/messaging/inbox?channel=general';
  // Everyone else uses the general channel inbox
  return '/messaging/inbox?channel=general';
}
