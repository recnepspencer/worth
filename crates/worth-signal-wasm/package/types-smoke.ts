import {
  clockCapability,
  type ControllerContract,
  createSignals,
  type FormActionPlan,
  type FormController,
  type FormValidationArtifact,
  type GraphMutationRequest,
  type GraphPublicationRequest,
  hostCapabilityPlan,
  onlineCapability,
  persistenceCapability,
  type PublishedGraphTransaction,
  type PublishedSignalGraph,
  type ScopedSignalNamespace,
  type SignalNamespace,
  resourceParamIdentity,
  resourceParams,
  viewportCapability,
  type ComputedSpec,
  type InputSignalHandle,
  type OutputSpec,
  type Signal,
  visibilityCapability,
} from "./index.js";

let visibilityState: "visible" | "hidden" = "visible";
let viewportState = { width: 1280, height: 720 };
let onlineState: "online" | "offline" = "online";
let clockTick = 0;
let persistedDraft = { mode: "draft", revision: 1 };
type ShippingOption = { id: string; label: string };

const signals = await createSignals({
  deployment: "mainThreadCompatibility",
  hostCapabilities: hostCapabilityPlan({
    visibility: visibilityCapability({
      source: {
        current() {
          return visibilityState;
        },
        subscribe() {
          return () => {};
        },
      },
      compatibility: "LiveOnly",
    }),
    viewport: viewportCapability({
      source: {
        current() {
          return viewportState;
        },
        subscribe() {
          return () => {};
        },
      },
    }),
    online: onlineCapability({
      source: {
        current() {
          return onlineState;
        },
        subscribe() {
          return () => {};
        },
      },
    }),
    clock: clockCapability({
      source: {
        current() {
          return clockTick;
        },
      },
      pollMs: 5,
    }),
    persistence: persistenceCapability({
      source: {
        current() {
          return persistedDraft;
        },
      },
    }),
  }),
});

const count: InputSignalHandle<number> = signals.input(1, { debugName: "count" });
const countDebugName: string | null = count.debugName;
const countResetCommit = count.reset();
const nameInput: InputSignalHandle<string> = signals.input("Ada");
const namedSpecInput: InputSignalHandle<string> = signals.spec.input("name", "Ada", { debugName: "name" });
const shippingOptions = signals.input([
  { id: "ground", label: "Ground" },
  { id: "air", label: "Air" },
], { debugName: "shippingOptions" });
const firstShippingOption = signals.linked(() => shippingOptions()[0], {
  debugName: "firstShippingOption",
});
const preservedShippingOption = signals.linked<ShippingOption[], ShippingOption>({
  source: () => shippingOptions(),
  computation: (options, previous) => (
    options.find((option) => option.id === previous?.value?.id) ?? options[0]
  ),
  debugName: "preservedShippingOption",
});
const linkedRelinkCommit = preservedShippingOption.relink();
const linkedResetCommit = preservedShippingOption.reset();
// @ts-expect-error linked app lane must not accept explicit ids
signals.linked(() => 1, { id: "count" });
const scopedSignals: ScopedSignalNamespace = signals.scope("itemDetail");
const nestedScopedSignals: ScopedSignalNamespace = scopedSignals.scope("editSession");
const scopedDescriptor = nestedScopedSignals.descriptor();
const scopedCanonicalCountId = nestedScopedSignals.canonicalId("count");
const scopedIdentity = nestedScopedSignals.signalIdentity("count");
const scopedIdentityGraphId = scopedIdentity.graphId;
const scopedIdentityRootScopeId = scopedIdentity.rootScopeId;
const scopedIdentityScopePath = scopedIdentity.scopePath;
const scopedDescriptorPath = scopedDescriptor.path;
const scopedDescriptorIdentity = scopedDescriptor.identity;
const scopedDescriptorGraphOwnerId = scopedDescriptor.graphOwnerId;
const asyncRootInput = await signals.inputAsync({
  title: "Ship docs",
  done: false,
});
const asyncScopedInput = await scopedSignals.inputAsync(3);
const asyncRootComputed = await signals.computedAsync<number>({
  reads: [asyncRootInput.id],
  expr: {
    kind: "value",
    value: 1,
  },
  identity: { kind: "exact" },
});
const asyncRootComputedCallback = await signals.computedAsync<number>(
  () => asyncScopedInput() + 1,
);
const asyncScopedOutput = await scopedSignals.outputAsync<{ total: number }>({
  reads: [asyncScopedInput.id, asyncRootComputed.id],
  expr: {
    kind: "object",
    fields: [
      ["total", { kind: "read", id: asyncRootComputed.id }],
    ],
  },
  identity: { kind: "exact" },
});
const asyncScopedOutputCallback = await scopedSignals.outputAsync<{ total: number }>(
  () => ({ total: asyncRootComputedCallback() }),
);
const asyncLinked = await signals.linkedAsync(() => asyncRootInput());
await asyncLinked.relink();
const asyncScopedLinked = await scopedSignals.linkedAsync({
  source: () => asyncRootInput(),
  computation: (value: { title: string; done: boolean }) => value,
});
await asyncScopedLinked.reset();
const workerFirstDetailFamily = signals.resource.detail({
  params: resourceParams(),
  normalizeParams: ({ taskId }: { taskId: string }) =>
    resourceParamIdentity({ taskId }, taskId),
  load: ({ taskId }: { taskId: string }) => ({
    id: taskId,
    title: "Draft",
  }),
});
const workerFirstDetailLine = workerFirstDetailFamily.line({ taskId: "task-1" } as never);
const workerFirstCollectionFamily = scopedSignals.resource.collection({
  params: resourceParams(),
  normalizeParams: ({ workspaceId }: { workspaceId: string }) =>
    resourceParamIdentity({ workspaceId }, workspaceId),
  itemIdentity: (item: { id: string }) => item.id,
  load: ({ workspaceId }: { workspaceId: string }) => [{
    id: `${workspaceId}:1`,
    title: "Scoped",
  }],
});
const workerFirstPagedFamily = signals.resource.paged({
  params: resourceParams(),
  normalizeParams: ({ feedId }: { feedId: string }) =>
    resourceParamIdentity({ feedId }, feedId),
  itemIdentity: (item: { id: string }) => item.id,
  accumulatePage: (existing: { id: string }[], next: { id: string }[]) => [...existing, ...next],
  load: ({ feedId }: { feedId: string }) => [{ id: `${feedId}:1`, title: "Paged" }],
});
const workerFirstExternalDetail = signals.resource.compatibility.detail({
  version: "worth-resource-external-v1",
  family: "detail",
  definitionId: "external-detail",
  requestContract: "native-v1",
  reconciliationContract: "none",
  declaration: {
    params: resourceParams(),
    normalizeParams: ({ taskId }: { taskId: string }) =>
      resourceParamIdentity({ taskId }, taskId),
    load: ({ taskId }: { taskId: string }) => ({ id: taskId, title: "External" }),
  },
});
const workerFirstApi = signals.api({
  baseUrl: "https://example.test",
  effects: signals.resource.effects.branchNative(),
});
const workerFirstApiDetailLine = workerFirstApi.url("/tasks/:taskId").response(
  signals.resource.response.detail()(),
).detail({
  load: ({ taskId }: { taskId: string }) => ({ id: taskId, title: "API" }),
}).line({ taskId: "task-1" } as never);
const routes = signals.router.define({
  home: signals.router.route("/"),
  userDetail: signals.router.route("/users/:userId", {
    search: {
      tab: signals.router.search.optional.string(),
      page: signals.router.search.optional.number(),
      active: signals.router.search.optional.boolean(),
    },
    hash: signals.router.hash.string(),
  }),
});
const projectedAuthSource = signals.router.host.string("auth");
const projectedWorkspaceReadySource = signals.router.resource.boolean("workspaceReady");
const projectedTenantCapabilitySource = signals.router.graph.string("tenantCapability");
const projectedAuthRequired = signals.router.prerequisite("projected-auth-required", {
  consumes: [projectedAuthSource, projectedWorkspaceReadySource, projectedTenantCapabilitySource] as const,
  evaluate: ({ consume, allow, denied }) => (
    consume(projectedAuthSource) === "signedIn" &&
      consume(projectedWorkspaceReadySource) === true &&
      consume(projectedTenantCapabilitySource) === "granted"
      ? allow({ reason: "authenticated" })
      : denied({ reason: "admissionSourcesBlocked" })
  ),
});
const projectedRouteRecovery = signals.router.recovery(
  "projected-user-detail-recovery",
  ({ terminalArtifact, fallback }) => (
    terminalArtifact.kind === "notFound"
      ? fallback({ href: "/users", reason: "staleDetail" })
      : null
  ),
);
const projectedRouteLabel = signals.output(() => "detail");
const projectedDetailController = signals.controller({
  outputs: {
    projectedRouteLabel,
  },
});
const projectedDetailGraph = signals.graph("routeDetailGraph", {
  outputs: {
    projectedRouteLabel,
  },
});
const projectedRoutes = signals.router.define({
  app: signals.router.layout("/", { outlet: "shell" }, {
    home: signals.router.route("/"),
    users: signals.router.layout("/users", { outlet: "detail" }, {
      index: signals.router.route("/users"),
      detail: signals.router.route("/users/:userId", {
        search: {
          tab: signals.router.search.optional.string(),
        },
        controllers: {
          detail: projectedDetailController,
        },
        graphs: {
          detailGraph: projectedDetailGraph,
        },
        resources: {
          detailLine: signals.router.resourceLine(workerFirstDetailFamily, {
            params: ({ params }) => ({ taskId: params.userId }),
            prefetch: "hover",
          }),
        },
        admission: [projectedAuthRequired],
        recovery: [projectedRouteRecovery],
        forms: signals.router.forms("user-detail-form", {
          continuity: "preserve",
        }),
      }),
    }),
  }),
});
const routeBreadcrumbDeclaration = signals.router.breadcrumb({
  id: "user-detail",
  label: ({ params }) => `User ${params.userId}`,
  parent: signals.router.breadcrumbParent({
    fallback: signals.router.breadcrumbEntry({
      id: "users",
      label: "Users",
      target: "/users",
    }),
  }),
});
const routeBreadcrumbTrailDeclaration = signals.router.breadcrumbTrail([
  signals.router.breadcrumbEntry({
    id: "workspace",
    label: "Workspace",
    target: "/workspace/acme",
  }),
]);
const breadcrumbRoute = signals.router.route("/users/:userId", {
  breadcrumb: routeBreadcrumbDeclaration,
});
const breadcrumbRouteTree = signals.router.define({
  breadcrumbRoute,
});
const breadcrumbProjectedCandidate = breadcrumbRouteTree.project("/users/task-1");
const breadcrumbProjectedEntry = breadcrumbProjectedCandidate?.route().breadcrumb();
const breadcrumbProjectedTrail = breadcrumbProjectedCandidate?.route().breadcrumbTrail();
const scopedRoutes = scopedSignals.router.define({
  step: signals.router.route("/wizard/:stepId"),
});
const rawLocationAuthority = signals.router.raw(
  "/users/task-1?page=2&tab=activity&active=true#panel",
  { navigationType: "manual" },
);
const browserHistoryIngress = signals.router.browserHistory.push("/users/task-1?page=2", {
  routeIdentity: "userDetail:task-1",
  runtimeRouteSourceId: "routeIdentity",
  routeValue: "userDetail:task-1",
  runtimeContinuitySourceId: "routeContinuity",
  continuityValue: "restored",
});
const browserHistoryIngressKind:
  import("./types/router_surface.js").BrowserHistoryNavigationKind =
    browserHistoryIngress.navigationKind;
const browserHistoryIngressDigest: string =
  browserHistoryIngress.verification().browserHistoryEnvelopeDigest;
const browserHistoryWriteback = signals.router.browserHistory.writeback.replace(
  routes.userDetail.to({
    params: { userId: "task-1" },
    search: { page: 2 },
  }),
  {
    routeIdentity: "userDetail:task-1",
    runtimeRouteSourceId: "routeIdentity",
    routeValue: "userDetail:task-1",
  },
);
const browserHistoryWritebackDigest: string =
  browserHistoryWriteback.verification().browserHistoryWritebackDigest;
const browserHistoryExternalWriteback = signals.router.browserHistory.writeback.external(
  "https://example.com/docs/router",
);
const browserHistoryStory = signals.router.browserHistory.story();
const browserHistoryBreadcrumbTrail = browserHistoryStory.breadcrumbTrail();
const carriedBreadcrumbs = signals.router.carryBreadcrumbs(browserHistoryBreadcrumbTrail.entries);
const carriedBreadcrumbsDigest: string =
  carriedBreadcrumbs.verification().carriedBreadcrumbsDigest;
const rawLocationNavigationType:
  import("./types/router_surface.js").RawLocationNavigationType =
    rawLocationAuthority.navigationType;
const canonicalUrlAuthority = rawLocationAuthority.canonical();
const canonicalUrlAuthorityHref: string = canonicalUrlAuthority.href;
const canonicalUrlAuthorityDigest: string =
  canonicalUrlAuthority.verification().canonicalUrlDigest;
const userDetailLocation = routes.userDetail.to({
  params: { userId: "task-1" },
  search: { tab: "activity", page: 2, active: true },
  hash: "panel",
});
const routeSequenceScenario = routes.simulateSequence([
  routes.home.to(),
  userDetailLocation,
]);
const routeSequenceResult = await routeSequenceScenario.run();
routeSequenceResult.steps;
routeSequenceResult.story;
routeSequenceResult.replay.outcomes();
routeSequenceResult.replay.breadcrumbTrail();
routeSequenceResult.replay.currentEntries();
const routeSequenceFirstOutcomeKind: string = routeSequenceResult.replay.outcomes()[0].kind;
const userDetailCanonical = routes.userDetail.canonical({
  params: { userId: "task-1" },
  search: { tab: "activity", page: 2, active: true },
  hash: "panel",
});
const userDetailCanonicalHref: string = userDetailCanonical.href;
const userDetailCanonicalDigest: string = userDetailCanonical.equivalenceDigest;
const userDetailReferenceVerification = routes.userDetail.verification();
const userDetailRouteSchemaDigest: string = userDetailReferenceVerification.routeSchemaDigest;
const userDetailCanonicalVerification = userDetailCanonical.verification();
const userDetailCanonicalUrlDigest: string =
  userDetailCanonicalVerification.canonicalUrlDigest;
const userDetailIntent = routes.userDetail.intent(
  {
    params: { userId: "task-1" },
    search: { tab: "activity", page: 2, active: true },
    hash: "panel",
  },
  {
    kind: "replace",
  },
);
const userDetailIntentDescriptorKind = userDetailIntent.descriptor().kind;
const userDetailIntentCanonicalDigest: string =
  userDetailIntent.descriptor().canonical().equivalenceDigest;
const userDetailIntentVerificationDigest: string =
  userDetailIntent.verification().navigationIntentDigest;
const userDetailPlan = userDetailIntent.policy({
  continuity: "preserve-visible-while-pending",
  projectionRefresh: "explicit",
  artifactPolicy: "diagnostics",
  deployment: "workerFirst",
}).compile();
const userDetailPlanKind = userDetailPlan.kind;
const userDetailPlanHref: string = userDetailPlan.href;
const userDetailPlanCost = userDetailPlan.cost();
const userDetailPlanLooksExpensive: boolean = userDetailPlanCost.looksExpensive;
const userDetailPlanProjectionRefresh = userDetailPlan.projectionPolicy().projectionRefresh;
const userDetailPlanCanonicalDigest: string = userDetailPlan.canonical().equivalenceDigest;
const userDetailPlanExplainCanonicalDigest: string =
  userDetailPlan.explain().canonical.equivalenceDigest;
const userDetailPlanVerificationDigest: string =
  userDetailPlan.verification().navigationPlanDigest;
const userDetailPlanExplainabilityDigest: string =
  userDetailPlan.verification().navigationExplainabilityDigest;
const directUserDetailPlan = userDetailLocation.plan({
  continuity: "refresh-immediately",
  projectionRefresh: "immediate",
});
const directUserDetailPlanKind = directUserDetailPlan.kind;
const directUserDetailPlanExplanationHref: string = directUserDetailPlan.explain().href;
const directUserDetailCanonicalDigest: string =
  userDetailLocation.canonical().canonicalUrlDigest;
const userDetailHref: string = routes.userDetail.href({
  params: { userId: "task-1" },
  search: { tab: "activity", page: 2, active: true },
  hash: "panel",
});
const matchedUserDetail = routes.userDetail.match(userDetailHref);
const matchedUserDetailId: string | number | undefined = matchedUserDetail?.params.userId;
const matchedUserDetailPage: number | undefined = matchedUserDetail?.search.page;
const matchedUserDetailActive: boolean | undefined = matchedUserDetail?.search.active;
const matchedUserDetailHash: string | undefined = matchedUserDetail?.hash;
const matchedFromRawAuthority = routes.userDetail.match(
  signals.router.raw("/users/task-1?active=true&page=2&tab=activity#panel"),
);
const matchedFromCanonicalAuthority = routes.userDetail.match(
  signals.router.canonical("/users/task-1?active=true&page=2&tab=activity#panel"),
);
const projectedCandidate = projectedRoutes.project("/users/task-1?tab=activity");
const projectedTreeWarmup = projectedRoutes.warmup("/users/task-1?tab=activity", "intent");
const projectedRoute = projectedCandidate?.route();
const projectedRouteId: string | undefined = projectedRoute?.routeId;
const projectedRouteParamId: string | number | undefined = projectedRoute?.params.userId;
const projectedRouteCanonicalDigest: string | undefined =
  projectedRoute?.canonical().equivalenceDigest;
const projectedRouteResourceNames: ReadonlyArray<string> | undefined =
  projectedRoute?.resourceNames();
const projectedRoutePrefetchPosture:
  import("./types/router_surface.js").RouteResourcePrefetchPosture | undefined =
    projectedRoute?.resource("detailLine").prefetchPosture();
const projectedRoutePrefetchDigest: string | undefined =
  projectedRoute?.resource("detailLine").prefetch().verification().routeResourcePrefetchDigest;
const projectedRouteWarmupDigest: string | undefined =
  projectedRoute?.resource("detailLine").warmup("intent").verification().routeResourcePrefetchDigest;
const projectedCandidatePrefetch = projectedCandidate?.prefetch("hover");
const projectedCandidateWarmup = projectedCandidate?.warmup("intent");
const projectedCandidatePrefetchTrigger:
  import("./types/router_surface.js").RoutePrefetchTrigger | undefined =
    projectedCandidatePrefetch?.trigger;
const projectedCandidatePrefetchResourceDigest: string | undefined =
  projectedCandidatePrefetch?.resource("detailLine").verification().routeResourcePrefetchDigest;
const projectedCandidateWarmupSkippedNames: ReadonlyArray<string> | undefined =
  projectedCandidateWarmup?.skippedResourceNames();
const projectedTreeWarmupDigest: string | undefined =
  projectedTreeWarmup?.verification().routePrefetchDigest;
const projectedWarmupIngress = signals.router.warmup.hover("/users/task-1", {
  sourceId: "sidebar-link",
  routeIdentity: "usersDetail",
});
const projectedWarmupIngressDigest: string =
  projectedWarmupIngress.verification().routeWarmupIngressDigest;
const projectedWarmupReport = projectedRoutes.applyWarmupIngress(projectedWarmupIngress);
const projectedWarmupBoundaryArtifact:
  "routeWarmupStarted" | "noMatchingWarmupResources" | "noProjectedCandidate" =
    projectedWarmupReport.diagnostics().boundaryArtifact;
const projectedAdmissionPlan = projectedCandidate?.admission({
  auth: "signedIn",
  workspaceReady: true,
  tenantCapability: "granted",
});
const projectedAdmissionPlanDigest: string | undefined =
  projectedAdmissionPlan?.verification().admissionPlanDigest;
const projectedAdmissionRecoveryNames: ReadonlyArray<string> | undefined =
  projectedAdmissionPlan?.recoveryNames();
const projectedAdmissionPlanProvenanceAttemptedRouteId: string | undefined =
  projectedAdmissionPlan?.provenance().attemptedRouteId;
const projectedAdmissionPlanConsumedSourceName: string | undefined =
  projectedAdmissionPlan?.provenance().consumedSources[0]?.name;
const projectedAdmissionOutcome = await projectedRoutes.admit("/users/task-1?tab=activity", {
  auth: "signedIn",
  workspaceReady: true,
  tenantCapability: "granted",
});
const projectedBrowserHistoryReport = await projectedRoutes.admitBrowserHistoryIngress(
  signals.router.browserHistory.push("/users/task-1?tab=activity", {
    routeIdentity: "userDetail:task-1",
  }),
  {
    auth: "signedIn",
    workspaceReady: true,
    tenantCapability: "granted",
  },
);
const projectedAdmissionOutcomeKind = projectedAdmissionOutcome.kind;
const projectedTransition = projectedAdmissionOutcome.kind === "admitted"
  ? await projectedRoutes.transition(projectedAdmissionOutcome, projectedCandidatePrefetch!, {
    continuity: "preserve-visible-while-pending",
  })
  : null;
const projectedTransitionVisibleSource:
  import("./types/router_surface.js").RouteVisibleChangeSource | undefined =
    projectedTransition?.diagnostics().visibleChangeSource;
const projectedTransitionDigest: string | undefined =
  projectedTransition?.verification().routeTransitionDigest;
const projectedBrowserHistoryEnvelopeFamily = projectedBrowserHistoryReport.envelopeFamily;
const projectedBrowserHistoryRouteTruthDigest: string =
  projectedBrowserHistoryReport.verification().routeTruthDigest;
const projectedBrowserHistoryWritebackReport = await projectedRoutes.applyBrowserHistoryWriteback(
  browserHistoryWriteback,
  {
    auth: "signedIn",
    workspaceReady: true,
    tenantCapability: "granted",
  },
);
const projectedBrowserHistoryWritebackFamily = projectedBrowserHistoryWritebackReport.envelopeFamily;
const projectedBrowserHistoryWritebackBoundaryDigest: string =
  projectedBrowserHistoryWritebackReport.verification().boundaryStoryDigest;
const projectedExternalWritebackReport = await projectedRoutes.applyBrowserHistoryWriteback(
  browserHistoryExternalWriteback,
);
const projectedExternalWritebackOutcome = projectedExternalWritebackReport.outcome();
const projectedBrowserHistorySeedEvent = browserHistoryStory.record(projectedBrowserHistoryReport);
const projectedBrowserHistoryEvent = browserHistoryStory.record(projectedBrowserHistoryWritebackReport);
const projectedBrowserHistoryEvents = browserHistoryStory.events();
const browserHistoryStoryCurrent = browserHistoryStory.current();
const browserHistoryStoryLatestBoundaryEvent = browserHistoryStory.latestBoundaryEvent();
const browserHistoryStoryCurrentRouteTruthEvent = browserHistoryStory.currentRouteTruthEvent();
const projectedBrowserHistoryBack = browserHistoryStory.back();
const projectedBrowserHistoryBreadcrumbs = browserHistoryStory.breadcrumbs();
const projectedBrowserHistoryBackProvenance = browserHistoryStory.backProvenance();
const projectedBrowserHistoryBreadcrumbTrail = browserHistoryStory.breadcrumbTrail();
const projectedBrowserHistoryStoryDigest: string =
  browserHistoryStory.verification().historyStoryDigest;
const projectedAdmissionOutcomeRecovery = projectedAdmissionOutcome.recovery();
const projectedAdmissionDiagnosticsRecovery = projectedAdmissionOutcome.diagnostics().recovery;
const projectedAdmissionOutcomeProvenance = projectedAdmissionOutcome.provenance();
const projectedAdmissionOutcomeTerminalSource = projectedAdmissionOutcome.provenance().terminalSource;
const projectedAdmissionOutcomeRecoveryTrail = projectedAdmissionOutcome.provenance().recoveryTrail;
const projectedAdmissionFormsAuthority =
  projectedAdmissionOutcome.kind === "admitted"
    ? projectedAdmissionOutcome.route().formsAuthority()
    : null;
const projectedAdmissionFormsAuthoritySurfaceId: string | null =
  projectedAdmissionFormsAuthority?.surfaceId ?? null;
const projectedAdmissionFormsAuthorityDigest: string | null =
  projectedAdmissionFormsAuthority?.verification().formsAuthorityDigest ?? null;
const projectedAdmissionRouteResourceFamilyId: string | null =
  projectedAdmissionOutcome.kind === "admitted"
    ? projectedAdmissionOutcome.route().resource("detailLine").current().descriptor.family.familyId
    : null;
const projectedAdmissionRouteResourceCanonicalKey: string | null =
  projectedAdmissionOutcome.kind === "admitted"
    ? projectedAdmissionOutcome.route().resource("detailLine").line().descriptor().canonicalParams.canonicalKey
    : null;
const projectedAdmissionOutcomeConsumedSourceFamily:
  import("./types/router_surface.js").RouteAdmissionSourceFamily | undefined =
    projectedAdmissionOutcome.provenance().prerequisiteDecisions[0]?.consumedSources[0]?.family;
if (projectedAdmissionOutcome.kind === "admitted") {
  projectedAdmissionOutcome.route().canonical();
} else {
  projectedAdmissionOutcome.artifact().reason;
}
const projectedLayouts = projectedCandidate?.layouts();
const projectedOutletId: string | null | undefined = projectedCandidate?.outlet().outletId;
const projectedCandidateDigest: string | undefined =
  projectedCandidate?.verification().projectedCandidateDigest;
const projectedAppOutletId: string = projectedRoutes.app.outletId;
const projectedUsersOutletId: string = projectedRoutes.app.users.outletId;
const projectedRouteAuthorityForm = signals.form({
  source: { title: "Ship docs" },
  fields: ({ field }) => ({
    title: field("title"),
  }),
  steps: ({ step }) => ({
    review: step("review", ["title"], { routeCoupled: true }),
  }),
  actions: ({ step }) => ({
    reviewRoute: step("reviewRoute", "review", "jump", { routeCoupled: true }),
  }),
});
if (projectedAdmissionOutcome.kind === "admitted" && projectedAdmissionOutcome.route().formsAuthority() !== null) {
  projectedRouteAuthorityForm.reportRouteAuthority(projectedAdmissionOutcome.route().formsAuthority()!);
}
const projectedRouteAuthoritySummary = projectedRouteAuthorityForm.routeAuthority().summary;
const projectedRouteAuthorityContinuity = projectedRouteAuthoritySummary.continuity;
const projectedRouteAuthorityHandoff = projectedRouteAuthoritySummary.handoff;
const projectedRouteAuthorityDraftContinuity = projectedRouteAuthoritySummary.draftContinuity;
const projectedRouteAuthorityDraftResolution:
  | "preservedValue"
  | "preservedFrozenValue"
  | "replacedFromSource"
  | "awaitingAdmittedTruth"
  | "authorityCleared"
  | undefined = projectedRouteAuthorityDraftContinuity?.draftResolution;
const projectedRouteAuthorityRouteCoupledBehavior:
  "admitted" | "deferred" | "cleared" | undefined = projectedRouteAuthorityHandoff?.routeCoupledBehavior;
const projectedRouteAuthorityContinuityApplied = projectedRouteAuthoritySummary.continuityApplied;
const projectedRouteAuthorityTransitionKind = projectedRouteAuthoritySummary.transitionKind;
const projectedRouteAuthorityPreviousAuthorityDigest = projectedRouteAuthoritySummary.previousAuthorityDigest;
const projectedRouteAuthorityChangedReports = projectedRouteAuthorityForm.routeAuthority().counters.changedReports;
const projectedRouteAuthorityPreservedDraftUpdates =
  projectedRouteAuthorityForm.routeAuthority().counters.preservedDraftUpdates;
const scopedRouteHref: string = scopedRoutes.step.href({
  params: { stepId: 3 },
});
const routeLocationCheck: boolean = signals.router.isRouteLocation(userDetailLocation);
const rawLocationCheck: boolean = signals.router.isRawLocationAuthority(rawLocationAuthority);
const canonicalUrlCheck: boolean =
  signals.router.isCanonicalUrlAuthority(canonicalUrlAuthority);
// @ts-expect-error routes must start with /
signals.router.route("users/:userId");
// @ts-expect-error route params must satisfy declared path params
routes.userDetail.to({ params: {} });
// @ts-expect-error route search values must satisfy declared value kinds
routes.userDetail.to({ params: { userId: "task-1" }, search: { page: "2" } });
// @ts-expect-error undeclared search params must not leak past declaration boundaries
routes.userDetail.to({ params: { userId: "task-1" }, search: { extra: "nope" } });
// @ts-expect-error hash must satisfy the declared hash kind
routes.userDetail.to({ params: { userId: "task-1" }, hash: 2 });
// @ts-expect-error root route does not admit undeclared params
routes.home.to({ params: { anything: "nope" } });
// @ts-expect-error navigation intent kinds stay inside the declared vocabulary
routes.userDetail.intent({ params: { userId: "task-1" } }, { kind: "teleport" });
// @ts-expect-error navigation policy continuity stays inside the declared vocabulary
routes.userDetail.intent({ params: { userId: "task-1" } }).policy({ continuity: "maybe" });
// @ts-expect-error navigation policy projection refresh stays inside the declared vocabulary
userDetailLocation.plan({ projectionRefresh: "later" });
// @ts-expect-error route resources must be declared with signals.router.resourceLine(...)
signals.router.route("/bad-resource", { resources: { detail: workerFirstDetailFamily } });
signals.router.resourceLine(workerFirstDetailFamily, {
  params: () => ({ taskId: "task-1" }),
  // @ts-expect-error route resource prefetch posture stays inside the declared vocabulary
  prefetch: "later",
});
// @ts-expect-error route prefetch trigger must stay inside the declared vocabulary
projectedCandidate?.prefetch("manual");
// @ts-expect-error route warmup trigger must stay inside the declared vocabulary
projectedCandidate?.warmup("manual");
// @ts-expect-error router warmup ingress requires local href string or raw location authority
signals.router.warmup.hover({ href: "/users/task-1" });
// @ts-expect-error raw location navigation types stay inside the declared vocabulary
signals.router.raw("/users/task-1", { navigationType: "teleport" });
// @ts-expect-error browser history ingress requires a local href string or raw location authority
signals.router.browserHistory.push({ href: "/users/task-1" });
// @ts-expect-error route sequence targets reject arbitrary numbers
const invalidRouteSequenceTarget:
  import("./types/router_sequence_surface.js").RouterSequenceTarget = 42;
// @ts-expect-error local writeback rejects arbitrary href-shaped objects that are not typed route locations or raw location authority
signals.router.browserHistory.writeback.push({ href: "/users/task-1" });
// @ts-expect-error local writeback requires explicit routeIdentity authority
signals.router.browserHistory.writeback.replace("/users/task-1");
signals.router.browserHistory.writeback.external("/users/task-1");
// @ts-expect-error browser history story requires a real boundary report
signals.router.browserHistory.story().record({ envelopeFamily: "browserHistoryIngress" });
// @ts-expect-error layout declarations require nested route children
signals.router.layout("/", { outlet: "shell" });
// @ts-expect-error projected route candidates expose candidate truth rather than route-location navigation APIs
projectedCandidate?.route().plan({ projectionRefresh: "immediate" });
// @ts-expect-error projected route candidates must not expose admitted-only forms authority
projectedCandidate?.route().formsAuthority();
// @ts-expect-error admission declarations must be created with signals.router.prerequisite(...)
signals.router.route("/broken", { admission: [{ name: "bad" }] });
// @ts-expect-error prerequisite consumes entries must be declared router admission sources
signals.router.prerequisite("broken-consumes", { consumes: [{ name: "auth" }], evaluate: ({ allow }) => allow() });
signals.router.prerequisite("broken-consume-usage", {
  consumes: [projectedAuthSource] as const,
  evaluate: ({ consume, allow }) => {
    // @ts-expect-error prerequisite evaluation may only consume declared sources
    consume(projectedWorkspaceReadySource);
    return allow();
  },
});
// @ts-expect-error recovery declarations must be created with signals.router.recovery(...)
signals.router.route("/broken-recovery", { recovery: [{ name: "bad" }] });
// @ts-expect-error projected route candidates must not masquerade as admitted route outcomes
projectedCandidate?.route().artifact();
// @ts-expect-error canonical route artifacts stay branded and must not accept structural forgeries
const forgedCanonicalArtifact:
  import("./types/router_surface.js").CanonicalRouteArtifact<"/", Record<string, never>, null> = {};
// @ts-expect-error raw location authorities stay branded and must not accept structural forgeries
const forgedRawLocationAuthority:
  import("./types/router_surface.js").RawLocationAuthority = {};
// @ts-expect-error canonical url authorities stay branded and must not accept structural forgeries
const forgedCanonicalUrlAuthority:
  import("./types/router_surface.js").CanonicalUrlAuthority = {};
// @ts-expect-error route verification packages stay branded and must not accept structural forgeries
const forgedRouteVerificationPackage:
  import("./types/router_surface.js").RouteReferenceVerificationPackage = {};
// @ts-expect-error raw location verification packages stay branded and must not accept structural forgeries
const forgedRawLocationVerificationPackage:
  import("./types/router_surface.js").RawLocationVerificationPackage = {};
// @ts-expect-error canonical url verification packages stay branded and must not accept structural forgeries
const forgedCanonicalUrlVerificationPackage:
  import("./types/router_surface.js").CanonicalUrlVerificationPackage = {};
// @ts-expect-error canonical verification packages stay branded and must not accept structural forgeries
const forgedCanonicalVerificationPackage:
  import("./types/router_surface.js").CanonicalRouteVerificationPackage = {};
// @ts-expect-error navigation intent verification packages stay branded and must not accept structural forgeries
const forgedNavigationIntentVerificationPackage:
  import("./types/router_surface.js").NavigationIntentVerificationPackage = {};
// @ts-expect-error navigation plan verification packages stay branded and must not accept structural forgeries
const forgedNavigationPlanVerificationPackage:
  import("./types/router_surface.js").NavigationPlanVerificationPackage = {};
// @ts-expect-error projected route capabilities stay branded and must not accept structural forgeries
const forgedProjectedRouteCapability:
  import("./types/router_surface.js").ProjectedRouteCapability = {};
// @ts-expect-error projected route candidates stay branded and must not accept structural forgeries
const forgedProjectedRouteCandidate:
  import("./types/router_surface.js").ProjectedRouteCandidate = {};
const scopedCount = nestedScopedSignals.input(1, { debugName: "count" });
const scopedStringValue = nestedScopedSignals.input("value", { debugName: "scopedStringValue" });
const scopedDeclarativeDouble = nestedScopedSignals.computed({
  reads: [scopedCount.id],
  expr: {
    kind: "sum",
    args: [
      { kind: "read", id: scopedCount.id },
      { kind: "read", id: scopedCount.id },
    ],
  },
  identity: { kind: "exact" },
});
const scopedDeclarativeWhen = nestedScopedSignals.computed({
  reads: [scopedCount.id],
  when: {
    expr: {
      kind: "gt",
      left: { kind: "read", id: scopedCount.id },
      right: { kind: "value", value: 0 },
    },
  },
  expr: { kind: "read", id: scopedCount.id },
  identity: { kind: "exact" },
});
const scopedLabel = nestedScopedSignals.computed(() => `${scopedCount()}`, { debugName: "label" });
const scopedOutput = nestedScopedSignals.output({
  reads: [scopedDeclarativeDouble.id],
  expr: {
    kind: "object",
    fields: [["count", { kind: "read", id: scopedDeclarativeDouble.id }]],
  },
  identity: { kind: "exact" },
});
const scopedOutputWhen = nestedScopedSignals.output({
  reads: [scopedDeclarativeWhen.id],
  when: {
    expr: {
      kind: "gt",
      left: { kind: "read", id: scopedDeclarativeWhen.id },
      right: { kind: "value", value: 0 },
    },
  },
  expr: {
    kind: "object",
    fields: [["count", { kind: "read", id: scopedDeclarativeWhen.id }]],
  },
  identity: { kind: "exact" },
});
const scopedSpecCount = nestedScopedSignals.spec.input("count", 1);
const scopedSpecComputed: Signal<number> = nestedScopedSignals.computedSpec<number>("doubleCount", {
  reads: [scopedSpecCount.id],
  expr: {
    kind: "multiply",
    args: [
      { kind: "read", id: scopedSpecCount.id },
      { kind: "value", value: 2 },
    ],
  },
});
const scopedSpecOutput: Signal<{ count: number }> = nestedScopedSignals.outputSpec<{ count: number }>("scopedPanel", {
  reads: [scopedSpecCount.id],
  expr: {
    kind: "object",
    fields: [
      ["count", { kind: "read", id: scopedSpecCount.id }],
    ],
  },
});
const scopedSpecOutputCallback: Signal<string> = nestedScopedSignals.outputCallback(
  "scopedPanelCallback",
  () => `${scopedCount()}`,
);
const viewport = signals.host.viewport;
const visibility = signals.host.visibility;
const online = signals.host.online;
const clock = signals.host.clock;
const persistence = signals.host.persistence;
// @ts-expect-error host capability lifecycle stays framework-owned
viewport?.free();
// @ts-expect-error host capability lifecycle stays framework-owned
visibility?.free();
// @ts-expect-error host capability lifecycle stays framework-owned
online?.free();
// @ts-expect-error host capability lifecycle stays framework-owned
clock?.free();
// @ts-expect-error host capability lifecycle stays framework-owned
persistence?.free();
const next: number = count();
const alsoNext: number = count.get();
const commit = await count.set(next + alsoNext);
const asyncRootInputCommit = await asyncRootInput.assign({
  done: true,
});
const asyncScopedInputCommit = await asyncScopedInput.set(4);
// @ts-expect-error patch unavailable on primitive input
count.patch(4);
// @ts-expect-error patch unavailable on primitive input
count.assign({ value: 2 });
const objectState = signals.input({
  title: "Ship docs",
  done: false,
});
const objectPatchCommit = objectState.patch({
  done: true,
});
const objectAssignCommit = objectState.assign({
  title: "Write release notes",
});
const taskForm = signals.form({
  source: objectState,
  fields: ({ field }) => ({
    title: field<string>("title"),
    done: field<boolean, string>("done", {
      parse: (rawValue) => rawValue === "true",
    }),
  }),
  validation: ({ field, form }) => ({
    titleRequired: field<string>("title", (value, context): FormValidationArtifact => {
      // @ts-expect-error validators receive read views and cannot mutate fields
      context.field?.set("mutated");
      return value.length > 0
        ? { kind: "valid", field: "title", digest: value }
        : {
          kind: "invalid",
          field: "title",
          message: {
            code: "task.title.required",
            severity: "error",
            target: "title",
            audience: "user",
            visibility: "visible",
          },
        };
    }),
    titleAndDone: form("titleAndDone", ["title", "done"], () => ({
      kind: "pending",
      asyncValidationId: "task-title-done-check",
    })),
  }),
  availability: ({ field, action, control, group, section }) => ({
    titleAvailability: field("title", ["done"], (values, context) => {
      // @ts-expect-error availability contexts are read-only
      context.form.field("title").set("mutated");
      return values.done
        ? { state: "readonly", draftPolicy: "freeze" }
        : "enabled";
    }),
    submitAvailability: action("submit", ["done"], (values) => (
      values.done ? "enabled" : { state: "blocked", reason: "task must be done" }
    )),
    saveControlAvailability: control("save", ["done"], (values) => (
      values.done ? "enabled" : "disabled"
    )),
    detailsGroupAvailability: group("details", ["title"], ["done"], (values) => (
      values.done ? "enabled" : "blocked"
    )),
    completionSectionAvailability: section("completion", ["done"], ["done"], () => "enabled"),
  }),
  admission: ({ field, action }) => ({
    titleEdit: field("title", "edit", ["done"], (values, context) => {
      // @ts-expect-error admission contexts are read-only
      context.form.field("title").input("mutated");
      return values.done ? "admitted" : { posture: "denied", reason: "not done" };
    }),
    submitAdmission: action("submit", "submit", ["done"], (values) => (
      values.done
        ? "admitted"
        : {
          posture: "requiresApproval",
          actorDigest: "actor:reviewer",
          policyDigest: "policy:done",
        }
    )),
    signatureAdmission: action("submit", "signature", ["done"], (_values, context) => ({
      posture: "requiresSignature",
      actorDigest: "actor:signer",
      policyDigest: "policy:signature",
      sourceDigest: context.binding.sourceDigest,
      patchDigest: context.binding.patchDigest,
      schemaDigest: context.binding.schemaDigest,
    })),
  }),
  steps: ({ step }) => ({
    details: step("details", ["title"], {
      order: 1,
      group: "main",
    }),
    completion: step("completion", ["done"], {
      order: 2,
      dependencies: ["done"],
      resolve: (values, context) => {
        // @ts-expect-error step contexts expose read views, not mutable handles
        context.form.field("done").set(true);
        return values.done ? "active" : { posture: "blocked", reason: "task is not done" };
      },
    }),
  }),
  actions: ({ action, step }) => ({
    saveDraft: action("saveDraft", {
      patchPolicy: "allowEmpty",
      idempotency: "collapse",
      hostEffect: "draft.store",
    }),
    nextDetails: step("nextDetails", "details", "next"),
  }),
});
const taskFormController: FormController = taskForm;
const taskFormTitleValue: string = taskForm.fields.title.effectiveValue();
taskForm.fields.done.input("true").commitInput();
const taskFormDirty = taskForm.dirty().isDirty;
const taskFormDirtyComparedFields: number = taskForm.dirty().breadth.comparedFields;
const taskFormDirtyOmittedFields: number = taskForm.dirty().breadth.omittedFields;
const taskFormDirtyClearedFields: number = taskForm.dirty().breadth.clearedFields;
const taskFormDirtyEqualityCostBasis: string = taskForm.dirty().equality.costBasis;
const taskFormPatch = taskForm.patchPlan().operations[0]?.field ?? null;
const taskFormPatchComparedFields: number = taskForm.patchPlan().breadth.comparedFields;
const taskFormPatchSkippedRawInputFields: number = taskForm.patchPlan().breadth.skippedRawInputFields;
const taskFormPatchOmittedFields: number = taskForm.patchPlan().breadth.omittedFields;
const taskFormPatchClearedFields: number = taskForm.patchPlan().breadth.clearedFields;
const taskFormPatchEqualityCostBasis: string = taskForm.patchPlan().equality.costBasis;
const taskFormReady = taskForm.readiness().canSubmit;
const taskFormValidation = taskForm.validation().summary.pending;
const taskFormAvailability = taskForm.availability().summary.readonly;
const taskFormAvailabilityGroupCount = taskForm.availability().summary.byScope.group;
const taskFormAvailabilityDependencyReads = taskForm.availability().counters.dependencyReads;
const taskFormAvailabilityCostBasis = taskForm.availability().counters.costBasis;
const taskFormAvailabilityGroupField = taskForm.availability().artifacts[2]?.fields[0] ?? null;
const taskFormAdmission = taskForm.admission().summary.requiresApproval;
const taskFormAdmissionRegulatedCount = taskForm.admission().counters.regulatedArtifacts;
const taskFormAdmissionIncrementalStatus = taskForm.admission().counters.incrementalStatus;
const taskFormAdmissionBinding = taskForm.admission().artifacts[0]?.binding?.bindingDigest ?? null;
const taskFormAdmissionStale = taskForm.admission().artifacts[0]?.stale?.isStale ?? false;
const taskFormStepCount = taskForm.steps().summary.total;
const taskFormStepFieldMemberships = taskForm.steps().counters.stepFieldMemberships;
const taskFormStepUniqueMessages = taskForm.steps().counters.uniqueProjectedMessages;
const taskFormStepProgress = taskForm.steps().artifacts[0]?.progress ?? "blocked";
const taskFormActionPlan: FormActionPlan = taskForm.actionPlan("saveDraft");
const taskFormActionPlanDigest: string = taskFormActionPlan.planDigest;
const taskFormActionEffectDigest: string = taskFormActionPlan.proof.effectDigest;
const taskFormActionRecovery = taskFormActionPlan.recoveryActions[0]?.kind ?? null;
const taskFormActionRegulatedBinding =
  taskForm.actionPlan("submit").regulatedActionBindings[0]?.actionPlanDigest ?? null;
const taskFormActionDeniedCount = taskForm.actions().summary.denied;
const taskFormActionStepCount = taskForm.actions().counters.stepPlans;
const taskFormActionAttempt = taskForm.attemptAction("saveDraft");
const taskFormActionAttemptDigest: string = taskFormActionAttempt.resultDigest;
const taskFormActionHistoryCount: number = taskForm.actionHistory().length;
const taskFormActionExecution = taskForm.executeAction("saveDraft");
const taskFormActionExecutionSettlement = Promise.resolve(taskFormActionExecution)
  .then((execution) => execution.resultKind === "pending"
    ? taskForm.fulfillAction(execution.operationId, {
        reason: "type smoke settled",
        messages: [{
          code: "task.settled",
          scope: "action",
        }],
      })
    : execution);
const taskFormActionExecutionDigest: Promise<string> =
  Promise.resolve(taskFormActionExecution).then((execution) => execution.executionDigest);
const taskFormActionExecutionHistoryCount: number =
  taskForm.actionExecutionHistory().length;
const taskFormVerificationDigest: string = taskForm.verification().packageDigest;
const taskFormVerificationActionDigest: string =
  taskForm.verification().digests.actionCatalogDigest;
const taskFormVerificationPerformancePlans: number =
  taskForm.verification().performanceEnvelope.actions.plans;
const taskFormTitleWritePosture = taskForm.fieldWritePosture("title").canWrite;
const taskFormTitleDiagnosticsWritePosture =
  taskForm.fields.title.diagnostics().writePosture.canWrite;
const taskFormSubmitReady = taskForm.actionReadiness("submit").canRun;
const taskFormVisibleMessageCount = taskForm.visibleMessages().length;
const taskFormDiagnosticsSummary = taskForm.diagnosticsSummary();
const taskFormDiagnosticsRouteAuthorityDigest: string =
  taskFormDiagnosticsSummary.routeAuthority.digest;
const taskFormDiagnosticsRouteAuthorityPosture:
  "preserve" | "freeze" | "discard" | "defer" | "cleared" | null =
  taskFormDiagnosticsSummary.routeAuthority.handoff?.posture ?? null;
const taskFormDiagnosticsHistoryRouteAuthorityDigest: string =
  taskForm.diagnosticsHistory()[0]?.routeAuthorityDigest ?? "";
const taskFormDiagnosticsHistoryRouteAuthorityResolution:
  "preservedValue"
  | "preservedFrozenValue"
  | "replacedFromSource"
  | "awaitingAdmittedTruth"
  | "authorityCleared"
  | null =
  taskForm.diagnosticsHistory()[0]?.routeAuthorityDraftResolution ?? null;
const taskFormDiagnosticsRouteAuthorityAuditDigest: string =
  taskFormDiagnosticsSummary.routeAuthority.continuityAudit.digest;
const taskFormVerificationRouteAuthorityContinuityDigest: string =
  taskForm.verification().digests.routeAuthorityContinuityDigest;
const taskFormVerificationRouteAuthorityContinuityBehavior:
  "admitted" | "deferred" | "cleared" | null =
  taskForm.verification().routeAuthorityContinuity.routeCoupledBehavior;
const optionList = signals.input([
  { id: "draft", label: "Draft" },
  { id: "review", label: "Review" },
]);
// @ts-expect-error assign is restricted to plain object inputs
optionList.assign([{ id: "ready", label: "Ready" }]);
const viewportSize = viewport?.size() ?? { width: 0, height: 0 };
const viewportWidth = viewport?.width() ?? 0;
const viewportHeight = viewport?.height() ?? 0;
const viewportDescriptor = viewport?.descriptor();
const visibilityStateNow = visibility?.state() ?? "hidden";
const visibilityFlag = visibility?.isVisible() ?? false;
const visibilityDescriptor = visibility?.descriptor();
const onlineStateNow = online?.state() ?? "offline";
const onlineFlag = online?.isOnline() ?? false;
const onlineDescriptor = online?.descriptor();
const clockNow = clock?.now() ?? 0;
const clockDescriptor = clock?.descriptor();
const persistenceValue = (persistence?.value() ?? { mode: "draft", revision: 0 }) as {
  mode: "draft";
  revision: number;
};
const persistenceMode: "draft" = persistenceValue.mode;
const persistenceRevision: number = persistenceValue.revision;
const persistenceDescriptor = persistence?.descriptor();
const persistenceCommit = persistence?.commit();

const doubledSpec: ComputedSpec = {
  reads: ["count"],
  expr: {
    kind: "multiply",
    args: [
      { kind: "read", id: "count" },
      { kind: "value", value: 2 },
    ],
  },
};

const doubled: Signal<number> = signals.spec.computed<number>("doubled", doubledSpec, { debugName: "doubled" });
const doubledFromCallback: Signal<number> = signals.computed<number>(
  () => count() * 2,
  { debugName: "doubledCallback" },
);
const constantFromCallback: Signal<number> = signals.computed<number>(
  () => 2,
  { debugName: "constantCallback" },
);
const generatedFromCallback: Signal<number> = signals.computed<number>(() => 3, { debugName: "three" });
const gatedFromHostCapability: Signal<string> = signals.computed<string>(() => (
  visibility?.isVisible() ? "onscreen" : "hidden"
), { debugName: "gatedFromHostCapability" });
const viewportLabel: Signal<string> = signals.computed<string>(() => (
  `${viewport?.width() ?? 0}x${viewport?.height() ?? 0}`
), { debugName: "viewportLabel" });
const onlineLabel: Signal<string> = signals.computed<string>(() => (
  online?.isOnline() ? "online" : "offline"
), { debugName: "onlineLabel" });
const clockLabel: Signal<number> = signals.computed<number>(() => (
  (clock?.now() ?? 0) + count()
), { debugName: "clockLabel" });
const persistenceLabel: Signal<number> = signals.computed<number>(() => (
  persistence?.value().revision ?? 0
), { debugName: "persistenceLabel" });
const legacyDoubledFromSpecAlias: Signal<number> = signals.spec.computed<number>(
  "legacyDoubled",
  doubledSpec,
);

const panelSpec: OutputSpec = {
  reads: ["count", "doubled"],
  expr: {
    kind: "object",
    fields: [
      ["count", { kind: "read", id: "count" }],
      ["doubled", { kind: "read", id: "doubled" }],
    ],
  },
};

const panel = signals.spec.output<{ count: number; doubled: number }>("panel", panelSpec, { debugName: "panel" });
const graphDoubledHandle = signals.computed<number>(() => count() * 2, { debugName: "graphDoubled" });
const legacyPanelFromSpecAlias = signals.spec.output<{ count: number; doubled: number }>(
  "legacyPanel",
  panelSpec,
);
const snapshot = panel();
const panelSnapshotFromRead = signals.read<{ count: number; doubled: number }>(panel);
const countSnapshotFromRead = signals.read<number>(count);
const callbackPanel = signals.output<{ count: number; doubled: number }>(() => ({
  count: count(),
  doubled: doubled(),
}), { debugName: "callbackPanel" });
const callbackPanelSnapshot = callbackPanel();
const namespace: SignalNamespace = signals;
const graphRequest: GraphPublicationRequest<{
  count: InputSignalHandle<number>;
}, {
  count: InputSignalHandle<number>;
  doubled: typeof graphDoubledHandle;
  panel: typeof panel;
}> = {
  inputs: {
    count,
  },
  outputs: {
    count,
    doubled: graphDoubledHandle,
    panel,
  },
};
const graph: PublishedSignalGraph<typeof graphRequest.outputs, NonNullable<typeof graphRequest.inputs>> = signals.graph(
  "itemDetail",
  graphRequest,
);
const graphInputByName = graph.input("count");
const graphCount = graph.outputs.count();
const graphDoubled = graph.outputs.doubled();
const graphPanel = graph.outputs.panel();
const graphDescriptorKind = graph.descriptors()[0]?.publicationKind ?? null;
const graphInputDescriptor = graph.inputDescriptors()[0]?.sourceId ?? null;
const graphInputSnapshot = graph.readInputs();
const graphInputCountValue = graphInputSnapshot.count;
const graphOperationalContract = graph.operationalContract();
const graphOperationalWriteId = graphOperationalContract.writes.count;
const graphOperationalPatchCount = Object.keys(graphOperationalContract.patches).length;
const graphOperationalAuthority = graphOperationalContract.authorities.count.authority;
const graphOperationRequest: GraphMutationRequest<NonNullable<typeof graphRequest.inputs>> = {
  writes: {
    count: 2,
  },
  commands: {},
  reset: ["count"],
};
const graphWriteCommit = graph.writeInputs({
  count: 3,
});
const graphSingleWriteCommit = graph.writeInput("count", 6);
const graphPatchCommit = graph.patchInputs({});
// @ts-expect-error primitive graph input cannot be patched
graph.patchInput("count", {});
const graphResetCommit = graph.resetInputs(["count"]);
const graphSingleResetCommit = graph.resetInput("count");
const graphApplyCommit = graph.apply(graphOperationRequest);
const graphTransactionCommit = graph.transaction((
  tx: PublishedGraphTransaction<NonNullable<typeof graphRequest.inputs>>,
) => {
  tx.set("count", 4);
  tx.set(graph.inputs.count, 5);
  // @ts-expect-error primitive graph inputs must not admit graph transaction patch helpers
  tx.patch("count", 6);
});
const awaitedGraphTransactionCommit = await graph.transaction((
  tx: PublishedGraphTransaction<NonNullable<typeof graphRequest.inputs>>,
) => {
  tx.set("count", 8);
});
const graphTransactionAsyncCommit = await graph.transactionAsync((
  tx: PublishedGraphTransaction<NonNullable<typeof graphRequest.inputs>>,
) => {
  tx.set("count", 4);
});
const graphBatchAsyncCommit = await graph.batchAsync((
  tx: PublishedGraphTransaction<NonNullable<typeof graphRequest.inputs>>,
) => {
  tx.set(graph.inputs.count, 5);
});
const graphSnapshot = graph.read();
const graphCountValue = graphSnapshot.count;
const graphDoubledValue = graphSnapshot.doubled;
const graphPanelValue = graphSnapshot.panel;
const graphWhy = graph.why("count");
const graphReplay = graph.replayFor("doubled");
const graphLineage = graph.lineageFor("panel");
const graphReadVersions = graph.readVersions();
const graphPublicationSummary = graph.summary();
const graphDiagnosticsSurface = graph.inspectDiagnostics();
const graphHistorySurface = graph.inspectHistory();
const graphCompatibilityDefinition = graph.exportCompatibilityDefinition();
const graphExportDefinition = graph.exportDefinition();
const graphExportSnapshot = graph.exportSnapshot();
const graphImportPosture = graph.importPosture();
const graphCompatibilityCountId = graphCompatibilityDefinition.inputs.count;
const graphContract = graph.contract();
const graphContractDelta = graph.contractDelta(graphContract);
const graphContractHistory = graph.contractHistory();
const graphContractCountId = graphContract.inputs.count;
const graphDiagnosticsWhy = graphDiagnosticsSurface.outputs.count.why;
const graphDiagnosticsInputWhy = graphDiagnosticsSurface.inputs.count.why;
const graphDiagnosticsInputEntry = graphDiagnosticsSurface.input("count");
const graphDiagnosticsOutputEntry = graphDiagnosticsSurface.output("panel");
const graphDiagnosticsDependency = graphDiagnosticsSurface.dependenciesForOutput("panel");
const graphDiagnosticsContractSummary = graphDiagnosticsSurface.contractSummary();
const graphHistoryInputEntry = graphHistorySurface.input("count");
const graphHistoryOutputEntry = graphHistorySurface.output("panel");
const graphHistoryDependency = graphHistorySurface.dependenciesForOutput("panel");
const graphHistoryContractSummary = graphHistorySurface.contractSummary();
const graphDiagnosticsVersion = graphDiagnosticsSurface.outputs.panel.version;
const graphDiagnosticsInputVersion = graphDiagnosticsSurface.inputs.count.version;
const graphHistoryReplay = graphHistorySurface.outputs.doubled.replay;
const graphHistoryInputReplay = graphHistorySurface.inputs.count.replay;
const graphHistoryLineage = graphHistorySurface.outputs.panel.lineage;
const graphCompatibilityPanelId = graphCompatibilityDefinition.outputs.panel;
const graphCompatibilityContractCountId = graphCompatibilityDefinition.contract.inputs.count;
const graphCompatibilityRecipeId = graphCompatibilityDefinition.definitions.recipes[0]?.id ?? null;
const graphDiagnostics = graph.diagnostics();
const graphHistory = graph.history();
const graphSpecialist = graph.specialist();
const graphAdapters = graph.adapters();
const graphOutputByName = graph.output("panel");
const restoredGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).importGraph(graphExportDefinition, graphExportSnapshot);
await restoredGraph.ready();
const restoredGraphContract = restoredGraph.contract();
const restoredGraphContractHistory = restoredGraph.contractHistory();
const restoredGraphImportPosture = restoredGraph.importPosture();
const restoredGraphOperationalContract = restoredGraph.operationalContract();
const restoredGraphRead = restoredGraph.read();
const restoredGraphReadInputs = restoredGraph.readInputs();
const restoredGraphInputSignal = restoredGraph.input("count");
const restoredGraphWriteRunSummary = await restoredGraph.writeInput("count", 9);
const restoredGraphResetRunSummary = await restoredGraph.resetInput("count");
const restoredGraphHandleWriteRunSummary = await restoredGraph.inputs.count.set(10);
const restoredGraphHandleResetRunSummary = await restoredGraphInputSignal.reset();
const restoredGraphApplyRunSummary = await restoredGraph.apply({ writes: { count: 11 } });
const restoredGraphCompatibility = restoredGraph.exportCompatibilityDefinition();
const restoredGraphDiagnostics = restoredGraph.inspectDiagnostics();
const restoredGraphHistory = restoredGraph.inspectHistory();

function createEditSessionController(namespace: SignalNamespace) {
  return namespace.controller(({ input, linked, computed }) => {
    const serverItemData = input<{
      workflow_target_state_id?: number | null;
    } | null>(null);
    const draftEdits = input<{
      workflow_target_state_id?: number | null;
    }>({});

    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }));

    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }));
    const preferredTransition = linked<(number | null)[], number | null>({
      source: () => [null, serverItemData()?.workflow_target_state_id ?? null],
      computation: (options, previous) => (
        options.find((option) => option === previous?.value) ?? options[0]
      ),
    });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
        preferredTransition,
      },
    };
  });
}

function createWorkflowController(
  namespace: SignalNamespace,
  editSession: ReturnType<typeof createEditSessionController>,
) {
  return namespace.controller(({ computed }) => {
    const submitReadiness = computed(() => {
      const item = editSession.outputs.effectiveItemData();
      const dirty = editSession.outputs.dirtyState();

      return {
        enabled: dirty.isDirty && Boolean(item.workflow_target_state_id),
        targetStateId: item.workflow_target_state_id ?? null,
      };
    });

    return {
      outputs: {
        submitReadiness,
      },
    };
  });
}

const editSession = createEditSessionController(signals);
const workflow = createWorkflowController(signals, editSession);
const editSessionContract: ControllerContract = editSession;
const itemDetailGraph = signals.graph("itemDetailControllers", (builder) => {
  const scopedEditSession = builder.controller("editSession", ({ input, linked, computed }) => {
    const serverItemData = input<{
      workflow_target_state_id?: number | null;
    } | null>(null);
    const draftEdits = input<{
      workflow_target_state_id?: number | null;
    }>({});
    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }));
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }));
    const preferredTransition = linked<(number | null)[], number | null>({
      source: () => [null, serverItemData()?.workflow_target_state_id ?? null],
      computation: (options, previous) => (
        options.find((option) => option === previous?.value) ?? options[0]
      ),
    });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
        preferredTransition,
      },
    };
  });
  const scopedWorkflow = createWorkflowController(builder.scope("workflow"), scopedEditSession);
  return builder.expose({
    controllers: [scopedEditSession, scopedWorkflow],
  });
});
const scopedCounterGraph = signals.graph("scopedCounter", {
  outputs: {
    count: scopedCount,
    label: scopedLabel,
    panel: scopedOutput,
  },
});
const itemDetailGraphOutput = itemDetailGraph.output("submitReadiness");
const itemDetailGraphSummary = itemDetailGraph.summary();
const itemDetailGraphDiagnostics = itemDetailGraph.inspectDiagnostics();
const itemDetailGraphHistory = itemDetailGraph.inspectHistory();
const itemDetailGraphCompatibility = itemDetailGraph.exportCompatibilityDefinition();
const itemDetailGraphExportDefinition = itemDetailGraph.exportDefinition();
const itemDetailGraphExportSnapshot = itemDetailGraph.exportSnapshot();
const itemDetailGraphImportPosture = itemDetailGraph.importPosture();
const itemDetailGraphContract = itemDetailGraph.contract();
const itemDetailGraphContractDelta = itemDetailGraph.contractDelta(itemDetailGraphContract);
const itemDetailGraphContractHistory = itemDetailGraph.contractHistory();
const itemDetailGraphInput = itemDetailGraph.input("serverItemData");
const itemDetailGraphInputs = itemDetailGraph.readInputs();
const itemDetailGraphInputDescriptor = itemDetailGraph.inputDescriptors()[0]?.sourceId ?? null;
const itemDetailGraphCompatibilityInputId =
  itemDetailGraphCompatibility.inputs.serverItemData;
const itemDetailGraphCompatibilityContractInputId =
  itemDetailGraphCompatibility.contract.inputs.serverItemData;
const itemDetailGraphCompatibilityOutputId =
  itemDetailGraphCompatibility.outputs.submitReadiness;
const projectedComposedRoutes = signals.router.define({
  app: signals.router.layout("/", { outlet: "shell" }, {
    itemDetail: signals.router.route("/items/:itemId", {
      controllers: {
        editSession,
      },
      graphs: {
        itemDetailGraph,
      },
    }),
  }),
});
const projectedComposedCandidate = projectedComposedRoutes.project("/items/task-1");
const projectedComposedRoute = projectedComposedCandidate?.route();
const projectedComposedController = projectedComposedRoute?.controller("editSession");
const projectedComposedGraph = projectedComposedRoute?.graph("itemDetailGraph");
const projectedComposedOutlets = projectedComposedCandidate?.outlets();
const projectedComposedLeafOutlet = projectedComposedCandidate?.outlet();
const projectedComposedNestedOutlet = projectedComposedCandidate?.layouts()[0]?.outlet();
const projectedComposedNestedOccupant = projectedComposedNestedOutlet?.occupant();
const projectedComposedLeafOccupant = projectedComposedLeafOutlet?.occupant();
const projectedComposedControllerOutputs = projectedComposedController?.outputNames();
const projectedComposedGraphSummary = projectedComposedGraph?.summary();
const projectedComposedGraphOutputNames = projectedComposedGraph?.outputNames();
const projectedComposedCompositionDigest: string | undefined =
  projectedComposedCandidate?.verification().routeCompositionDigest;
const projectedComposedOutletStackDigest: string | undefined =
  projectedComposedCandidate?.verification().outletStackDigest;
if (projectedComposedNestedOccupant?.kind === "projectedLayoutPlacement") {
  projectedComposedNestedOccupant.capability();
}
if (projectedComposedLeafOccupant?.kind === "projectedRouteCapability") {
  projectedComposedLeafOccupant.controller("editSession");
}
// @ts-expect-error route references must not expose projected route-local controller access directly
routes.userDetail.controller("detail");
// @ts-expect-error route references must not expose projected route-local graph access directly
routes.userDetail.graph("detailGraph");
// @ts-expect-error projected controller capabilities stay on composition summaries rather than live controller handles
projectedComposedRoute?.controller("editSession").outputs.effectiveItemData;
// @ts-expect-error projected graph capabilities stay on composition summaries rather than live graph runtime methods
projectedComposedRoute?.graph("itemDetailGraph").output("submitReadiness");
// @ts-expect-error outlet consumers must narrow projected outlet occupants before using route-only APIs
projectedComposedCandidate?.outlets()[0]?.occupant().controller("editSession");
const itemDetailGraphContractInputId =
  itemDetailGraphContract.inputs.serverItemData;
const itemDetailGraphHistoryReplay = itemDetailGraphHistory.outputs.submitReadiness.replay;
const itemDetailGraphHistoryInputReplay = itemDetailGraphHistory.inputs.serverItemData.replay;
const itemDetailGraphDiagnosticsWhy = itemDetailGraphDiagnostics.outputs.submitReadiness.why;
const itemDetailGraphDiagnosticsInputWhy = itemDetailGraphDiagnostics.inputs.serverItemData.why;
const itemDetailGraphDependency =
  itemDetailGraphDiagnostics.dependenciesForOutput("submitReadiness");
const itemDetailGraphContractSummary = itemDetailGraphDiagnostics.contractSummary();
const itemDetailGraphHistoryDependency =
  itemDetailGraphHistory.dependenciesForOutput("submitReadiness");
const itemDetailGraphHistoryContractSummary = itemDetailGraphHistory.contractSummary();
const restoredItemDetailGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).importGraph(
  itemDetailGraphExportDefinition,
  itemDetailGraphExportSnapshot,
);
await restoredItemDetailGraph.ready();
const restoredItemDetailGraphContract = restoredItemDetailGraph.contract();
const restoredItemDetailGraphContractHistory = restoredItemDetailGraph.contractHistory();
const restoredItemDetailGraphImportPosture = restoredItemDetailGraph.importPosture();
const restoredItemDetailGraphRead = restoredItemDetailGraph.read();
const restoredItemDetailGraphReadInputs = restoredItemDetailGraph.readInputs();
const restoredItemDetailGraphPatchRunSummary =
  await restoredItemDetailGraph.inputs.serverItemData.patch({ workflow_target_state_id: 3 });
const restoredItemDetailGraphAssignRunSummary =
  await restoredItemDetailGraph.input("serverItemData").assign({ workflow_target_state_id: 4 });
const restoredItemDetailGraphDependency =
  restoredItemDetailGraph.inspectDiagnostics().dependenciesForOutput("submitReadiness");

function createFormController(namespace: SignalNamespace) {
  return namespace.controller(({ input, computed }) => {
    const serverValue = input<{
      id: string;
      title: string;
      status: string;
    }>({
      id: "task-7",
      title: "Ship docs",
      status: "draft",
    });
    const draftValue = input<{
      title?: string;
      status?: string;
    }>({
      title: "Ship docs",
      status: "ready",
    });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
    }));
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftValue()).length > 0,
    }));
    const validation = computed(() => ({
      titleMissing: !effectiveValue().title,
    }));

    return {
      inputs: {
        serverValue,
        draftValue,
      },
      outputs: {
        effectiveValue,
        dirtyState,
        validation,
      },
    };
  });
}

function createResourceController(
  namespace: SignalNamespace,
  form: ReturnType<typeof createFormController>,
) {
  return namespace.controller(({ input, computed }) => {
    const routeParams = input<{
      taskId: string;
      workspaceId: string;
    }>({
      taskId: "task-7",
      workspaceId: "alpha",
    });
    const resourceQuery = computed(() => ({
      taskId: routeParams().taskId,
      workspaceId: routeParams().workspaceId,
      status: form.outputs.effectiveValue().status,
    }));
    const submitAvailability = computed(() => ({
      enabled: form.outputs.dirtyState().isDirty && !form.outputs.validation().titleMissing,
      taskId: resourceQuery().taskId,
    }));

    return {
      inputs: {
        routeParams,
      },
      outputs: {
        resourceQuery,
        submitAvailability,
      },
    };
  });
}

function createAuthorityController(namespace: SignalNamespace) {
  return namespace.controller(({ input, computed, publicInput }) => {
    const serverValue = input({
      id: "task-7",
      title: "Ship docs",
    });
    const draftValue = input({
      title: "Ship docs",
    });
    const externalParams = input({
      taskId: "task-7",
    });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
      taskId: externalParams().taskId,
    }));

    return {
      inputs: {
        serverValue: publicInput(serverValue, { authority: "readOnly" }),
        draftValue: publicInput(draftValue),
        externalParams: publicInput(externalParams, { authority: "imported" }),
      },
      outputs: {
        effectiveValue,
      },
    };
  });
}

const taskEditorGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("taskEditor", (graph) => {
  const form = createFormController(graph.scope("form"));
  const resource = createResourceController(graph.scope("resource"), form);
  return graph.expose({
    controllers: [form, resource],
  });
});
const authorityGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("taskAuthority", (graph) => {
  const authority = createAuthorityController(graph.scope("authority"));
  return graph.expose({
    controllers: [authority],
  });
});
const requirednessGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("taskRequiredness", (graph) => {
  const scope = graph.scope("requiredness");
  const serverValue = scope.input({
    id: "task-7",
    title: "Ship docs",
  });
  const draftValue = scope.input({
    title: "Ship docs",
  });
  const effectiveValue = scope.computed(() => ({
    ...serverValue(),
    ...draftValue(),
  }));

  return graph.expose({
    inputs: {
      serverValue: graph.input.required(serverValue, { authority: "readOnly" }),
      draftValue: graph.input.optional(draftValue),
    },
    outputs: {
      effectiveValue,
    },
  });
});
const taskEditorGraphContract = taskEditorGraph.contract();
const taskEditorOperationalContract = taskEditorGraph.operationalContract();
const taskEditorContractDelta = taskEditorGraph.contractDelta(taskEditorGraphContract);
const taskEditorGraphInputId = taskEditorGraphContract.inputs.serverValue;
const taskEditorGraphOutputId = taskEditorGraphContract.outputs.submitAvailability;
const taskEditorPatchId = taskEditorOperationalContract.patches.draftValue;
const taskEditorGraphInputWhy = taskEditorGraph.inspectDiagnostics().inputs.serverValue.why;
const taskEditorGraphOutputReplay = taskEditorGraph.inspectHistory().outputs.submitAvailability.replay;
const taskEditorGraphDependency =
  taskEditorGraph.inspectDiagnostics().dependenciesForOutput("submitAvailability");
const taskEditorGraphContractSummary = taskEditorGraph.inspectDiagnostics().contractSummary();
const taskEditorGraphExportDefinition = taskEditorGraph.exportDefinition();
const taskEditorGraphExportSnapshot = taskEditorGraph.exportSnapshot();
const taskEditorGraphImportPosture = taskEditorGraph.importPosture();
const taskEditorGraphContractHistory = taskEditorGraph.contractHistory();
const restoredTaskEditorGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).importGraph(
  taskEditorGraphExportDefinition,
  taskEditorGraphExportSnapshot,
);
await restoredTaskEditorGraph.ready();
const restoredTaskEditorGraphHistory = restoredTaskEditorGraph.contractHistory();
const restoredTaskEditorGraphImportPosture = restoredTaskEditorGraph.importPosture();
const taskEditorGraphPatchCommit = taskEditorGraph.patchInputs({
  draftValue: {
    status: "published",
  },
});
const taskEditorGraphApplyCommit = taskEditorGraph.apply({
  writes: {
    routeParams: {
      taskId: "task-8",
      workspaceId: "beta",
    },
  },
  patches: {
    draftValue: {
      title: "Ship package",
    },
  },
  commands: {},
});
const taskEditorGraphTxCommit = taskEditorGraph.transaction((tx) => {
  tx.set("serverValue", {
    id: "task-7",
    title: "Ship docs",
    status: "ready",
  });
});
const authorityGraphContract = authorityGraph.contract();
const authorityGraphOperationalContract = authorityGraph.operationalContract();
const authorityGraphRead = authorityGraph.read();
const authorityGraphInputRead = authorityGraph.readInputs();
const authorityGraphReadOnlyAuthority =
  authorityGraphOperationalContract.authorities.serverValue.authority;
const authorityGraphImportedAuthority =
  authorityGraphOperationalContract.authorities.externalParams.authority;
const authorityGraphPatchId = authorityGraphOperationalContract.patches.draftValue;
const requirednessGraphDescriptors = requirednessGraph.inputDescriptors();
const requirednessServerRequiredness = requirednessGraphDescriptors[0]?.requiredness;
const requirednessDraftRequiredness = requirednessGraphDescriptors[1]?.requiredness;
const requirednessAuthorityRequiredness =
  requirednessGraph.operationalContract().authorities.serverValue.requiredness;
(await createSignals({ deployment: "mainThreadCompatibility" })).graph("invalidRequirednessTypes", (graph) => {
  const scope = graph.scope("requiredness");
  const value = scope.spec.input("value", 1);
  // @ts-expect-error contradictory requiredness must be unrepresentable
  const impossibleRequired = graph.input.required(value, { requiredness: "optional" });
  // @ts-expect-error contradictory requiredness must be unrepresentable
  const impossibleOptional = graph.input.optional(value, { requiredness: "required" });
  return graph.expose({
    inputs: {
      requiredValue: impossibleRequired,
      optionalValue: impossibleOptional,
    },
    outputs: {
      echoed: scope.output(() => value()),
    },
  });
});
const authorityGraphWriteCommit = authorityGraph.writeInputs({
  draftValue: {
    title: "Ready to ship",
  },
});
const authorityGraphPatchCommit = authorityGraph.patchInputs({
  draftValue: {
    title: "Approved",
  },
});
const authorityGraphResetCommit = authorityGraph.resetInputs(["draftValue"]);
const authorityGraphApplyCommit = authorityGraph.apply({
  writes: {
    draftValue: {
      title: "Ship package",
    },
  },
  commands: {},
});
const authorityGraphTransactionCommit = authorityGraph.transaction((tx) => {
  tx.set("draftValue", {
    title: "Queued",
  });
});
const explicitCallbackPanel = signals.spec.outputCallback<{ count: number; doubled: number }>(
  "callbackPanelExplicit",
  () => snapshot,
);
const workerFirstSignals = await createSignals();
const workerFirstExplicitComputed = workerFirstSignals.spec.computedCallback(
  "workerFirstExplicitComputed",
  () => 1,
);
const workerFirstScopedExplicitOutput = workerFirstSignals.scope("wizard").spec.outputCallback(
  "panelExplicit",
  () => ({ ok: true as const, count: workerFirstExplicitComputed() }),
);
const workerFirstExplicitComputedValue = workerFirstExplicitComputed();
const workerFirstScopedExplicitOutputValue = workerFirstScopedExplicitOutput();
const adapters = signals.adapters();
const definitions = adapters.exportDefinitions();
const runtimeEnvelope = adapters.exportRuntimeEnvelope();
await adapters.restoreExactRuntimeEnvelope(runtimeEnvelope);
const transportReport = adapters.hostCapabilityTransportReport(runtimeEnvelope);
const proof = adapters.runtimeProofReport();
const runtimeEnvelopeRestoreMode = runtimeEnvelope.runtimeEnvelopeRestoreMode;
const restoredBranchId = runtimeEnvelope.snapshot.snapshot.meta.branch_id;
const snapshotExplanationRetention =
  runtimeEnvelope.snapshot.snapshot.meta.artifact_retention.explanation_retention;
const checkpointImage = runtimeEnvelope.snapshot.snapshot.checkpoint_image;
const diagnosticGraph = runtimeEnvelope.snapshot.snapshot.diagnostic_graph;
const proofVersion = proof.proofSchemaVersion;
const proofDigest = proof.registryBundleDigest;
const maybeUnavailable = definitions.unavailableCallbacks.map(
  (artifact) => artifact.signalKind,
);
const diagnostics = signals.diagnostics();
const history = signals.history();
const specialist = signals.specialist();
const currentBranch = history.current_branch();
const branchReplay = history.replay_for_branch(currentBranch.id);
const branchSnapshot = history.branch_snapshot(currentBranch.id);
const branchEnvelope = history.branch_snapshot_envelope(currentBranch.id);
const branchSnapshotRestoreMode = branchSnapshot.snapshotRestoreMode;
const branchEnvelopeRestoreMode = branchEnvelope.snapshotEnvelopeRestoreMode;
history.restore_exact_snapshot(branchEnvelope);
history.restore_exact_branch_snapshot(currentBranch.id, branchSnapshot);
const branchProof = history.branch_state_proof(currentBranch.id);
const parityProof = history.replay_parity_proof(currentBranch.id, currentBranch.id);
const artifactProof = history.replay_artifact_proof({
  proofSchemaVersion: proof.proofSchemaVersion,
  registryBundleDigest: proof.registryBundleDigest,
  branchStateDigest: branchProof.stateDigest,
}, currentBranch.id);
const graphSummary = diagnostics.summaryNow();
const specialistGraphSummary = specialist.graphSummary();
const specialistEvaluateDirty = await specialist.evaluateDirty();
const performanceSummary = diagnostics.performanceSummary();
const latestFlow = diagnostics.latestFlow();
const latestObservation = diagnostics.latestObservation();
const latestHostCapabilityEvent = diagnostics.latestHostCapabilityEvent();
const recentHostCapabilityEvents = diagnostics.recentHostCapabilityEvents();
const hostCapabilityReport = diagnostics.hostCapabilityReport();
const hostCapabilityLineageDigest = hostCapabilityReport.lineageDigest;
const hostCapabilityBreadthDigest = hostCapabilityReport.breadthDigest;
const hostCapabilityLineageEntry = hostCapabilityReport.lineage[0] ?? null;
const hostCapabilityBreadthFamily = hostCapabilityReport.breadth.families[0] ?? null;
const latestFailure = diagnostics.latestFailure();
const latestInvalidationPlanningEstimate = diagnostics.latestInvalidationPlanningEstimate();
const recentHistory = diagnostics.recentHistory();
const latestHostCapabilityRead =
  latestFlow?.callbackNodes[0]?.hostCapabilityReads[0]?.compatibility ??
  latestObservation?.callbackNodes[0]?.hostCapabilityReads[0]?.compatibility ??
  null;
const unavailableHostCapabilityTransport =
  runtimeEnvelope.definitions.unavailableCallbacks[0]?.hostCapabilityTransports[0] ?? null;
const latestCallbackCurrentReads = latestFlow?.callbackNodes[0]?.currentReads ?? [];

const callbackNodeIds =
  latestFlow?.callbackNodes.map((node) => node.id) ??
  latestObservation?.callbackNodes.map((node) => node.id) ??
  [];
const latestHistoryNode = recentHistory[0]?.nodes[0]?.node ?? null;
const graphProfile = graphSummary.profile;
const specialistGraphProfile = specialistGraphSummary.profile;
const specialistTouchedNodes = specialistEvaluateDirty.touchedNodes;
const latestFailureMessage = latestFailure?.message ?? null;
const latestInvalidationPlanningSeedCount = latestInvalidationPlanningEstimate?.seed_count ?? 0;
const latestHostCapabilityEventKind = latestHostCapabilityEvent?.kind ?? null;
const latestHostCapabilityEventQueuedCount = latestHostCapabilityEvent?.queuedInvalidationCount ?? 0;
const latestHostCapabilityDeniedIds = latestHostCapabilityEvent?.deniedCallbackIds ?? [];
const hostCapabilityInvalidationCount = performanceSummary.hostCapabilityInvalidationCount ?? 0;
const hostCapabilityReadCount = performanceSummary.hostCapabilityReadCount ?? 0;
const hostCapabilityPollCount = performanceSummary.hostCapabilityPollCount ?? 0;
const hostCapabilityNoOpPollCount = performanceSummary.hostCapabilityNoOpPollCount ?? 0;
const hostCapabilityManualCommitCount = performanceSummary.hostCapabilityManualCommitCount ?? 0;
const hostCapabilityNoOpManualCommitCount =
  performanceSummary.hostCapabilityNoOpManualCommitCount ?? 0;
const hostCapabilityReevaluationCount = performanceSummary.hostCapabilityReevaluationCount ?? 0;
const hostCapabilityCompatibilityDenialCount =
  performanceSummary.hostCapabilityCompatibilityDenialCount ?? 0;
const hostCapabilityUnavailabilityArtifactCount =
  performanceSummary.hostCapabilityUnavailabilityArtifactCount ?? 0;
const hostCapabilityBroadFanoutDenialCount =
  performanceSummary.hostCapabilityBroadFanoutDenialCount ?? 0;
const branchReplayCallback = branchReplay.frames[0]?.callback?.registered ?? null;
const branchSnapshotBranchId = branchSnapshot.meta.branch_id;
const branchEnvelopeSnapshotId = branchEnvelope.snapshot.meta.snapshot_id;
const parityMismatchCount = parityProof.mismatchClasses.length;
const artifactParity = artifactProof.parity;

signals.transaction((tx) => {
  tx.set(count, snapshot.count + commit.touchedNodes);
  // @ts-expect-error primitive inputs must not admit transaction patch helpers
  tx.patch(count, 4);
  // @ts-expect-error computed handles must stay read-only inside transactions
  tx.set(doubled, 4);
});
const awaitedLegacyTransactionCommit = await signals.transaction((tx) => {
  tx.set(count, snapshot.count + commit.touchedNodes + 2);
});
const asyncTransactionCommit = await signals.transactionAsync((tx) => {
  tx.set(count, snapshot.count + commit.touchedNodes + 1);
});
const asyncAuthoredTransactionCommit = await signals.transactionAsync((tx) => {
  tx.patch(asyncRootInput, { title: "Close milestone" });
  tx.set(asyncScopedInput, 5);
});
const asyncBatchCommit = await signals.batchAsync((tx) => {
  tx.set(count, snapshot.count + asyncTransactionCommit.touchedNodes);
});

// @ts-expect-error branded callable handles must not accept structural forgeries
const forgedSignal: InputSignalHandle<number> = {
  id: "forged",
  get() {
    return 1;
  },
  set() {
    return commit;
  },
};

void asyncBatchCommit;
void graphTransactionAsyncCommit;
void graphBatchAsyncCommit;
void constantFromCallback;
void doubledFromCallback;
void generatedFromCallback;
void gatedFromHostCapability;
void viewportLabel;
void onlineLabel;
void clockLabel;
void persistenceLabel;
void legacyDoubledFromSpecAlias;
void legacyPanelFromSpecAlias;
void callbackPanelSnapshot;
void namespace;
void graph;
void graphCount;
void graphDoubled;
void graphPanel;
void graphSnapshot;
void graphCountValue;
void graphDoubledValue;
void graphPanelValue;
void graphDoubledHandle;
void graphDescriptorKind;
void graphInputByName;
void graphInputDescriptor;
void graphInputSnapshot;
void graphInputCountValue;
void graphOperationalContract;
void graphOperationalWriteId;
void graphOperationalPatchCount;
void graphOperationalAuthority;
void graphWriteCommit;
void graphPatchCommit;
void graphResetCommit;
void graphApplyCommit;
void graphTransactionCommit;
void graphWhy;
void graphReplay;
void graphLineage;
void graphReadVersions;
void graphPublicationSummary;
void graphCompatibilityDefinition;
void graphExportDefinition;
void graphExportSnapshot;
void graphImportPosture;
void graphCompatibilityCountId;
void graphContractHistory;
void graphCompatibilityPanelId;
void graphCompatibilityRecipeId;
void graphDiagnostics;
void graphHistory;
void graphSpecialist;
void graphAdapters;
void graphOutputByName;
void restoredGraph;
void restoredGraphContract;
void restoredGraphContractHistory;
void restoredGraphImportPosture;
void restoredGraphRead;
void restoredGraphReadInputs;
void restoredGraphCompatibility;
void restoredGraphDiagnostics;
void restoredGraphHistory;
void itemDetailGraph;
void itemDetailGraphOutput;
void itemDetailGraphSummary;
void itemDetailGraphCompatibility;
void itemDetailGraphExportDefinition;
void itemDetailGraphExportSnapshot;
void itemDetailGraphImportPosture;
void itemDetailGraphInput;
void itemDetailGraphInputs;
void itemDetailGraphInputDescriptor;
void itemDetailGraphCompatibilityInputId;
void itemDetailGraphCompatibilityOutputId;
void projectedComposedRoutes;
void projectedComposedCandidate;
void projectedComposedRoute;
void projectedComposedController;
void projectedComposedGraph;
void projectedComposedOutlets;
void projectedComposedLeafOutlet;
void projectedComposedNestedOutlet;
void projectedComposedNestedOccupant;
void projectedComposedLeafOccupant;
void projectedComposedControllerOutputs;
void projectedComposedGraphSummary;
void projectedComposedGraphOutputNames;
void projectedComposedCompositionDigest;
void projectedComposedOutletStackDigest;
void itemDetailGraphContractHistory;
void restoredItemDetailGraph;
void restoredItemDetailGraphContract;
void restoredItemDetailGraphContractHistory;
void restoredItemDetailGraphImportPosture;
void restoredItemDetailGraphRead;
void restoredItemDetailGraphReadInputs;
void restoredItemDetailGraphDependency;
void taskEditorOperationalContract;
void taskEditorPatchId;
void taskForm;
void taskFormController;
void taskFormTitleValue;
void taskFormDirty;
void taskFormDirtyComparedFields;
void taskFormDirtyOmittedFields;
void taskFormDirtyClearedFields;
void taskFormDirtyEqualityCostBasis;
void taskFormPatch;
void taskFormPatchComparedFields;
void taskFormPatchSkippedRawInputFields;
void taskFormPatchOmittedFields;
void taskFormPatchClearedFields;
void taskFormPatchEqualityCostBasis;
void taskFormReady;
void taskFormValidation;
void taskFormAvailability;
void taskFormAvailabilityGroupCount;
void taskFormAvailabilityDependencyReads;
void taskFormAvailabilityCostBasis;
void taskFormAvailabilityGroupField;
void taskFormAdmission;
void taskFormAdmissionRegulatedCount;
void taskFormAdmissionIncrementalStatus;
void taskFormAdmissionBinding;
void taskFormAdmissionStale;
void taskFormStepCount;
void taskFormStepFieldMemberships;
void taskFormStepUniqueMessages;
void taskFormStepProgress;
void taskFormActionPlan;
void taskFormActionPlanDigest;
void taskFormActionEffectDigest;
void taskFormActionRecovery;
void taskFormActionRegulatedBinding;
void taskFormActionDeniedCount;
void taskFormActionStepCount;
void taskFormActionAttempt;
void taskFormActionAttemptDigest;
void taskFormActionHistoryCount;
void taskFormActionExecution;
void taskFormActionExecutionDigest;
void taskFormActionExecutionSettlement;
void taskFormActionExecutionHistoryCount;
void taskFormVerificationDigest;
void taskFormVerificationActionDigest;
void taskFormVerificationPerformancePlans;
void taskFormTitleWritePosture;
void taskFormTitleDiagnosticsWritePosture;
void taskFormSubmitReady;
void taskFormVisibleMessageCount;
void taskFormDiagnosticsSummary;
void taskFormDiagnosticsRouteAuthorityDigest;
void taskFormDiagnosticsRouteAuthorityPosture;
void taskFormDiagnosticsHistoryRouteAuthorityDigest;
void taskFormDiagnosticsHistoryRouteAuthorityResolution;
void taskFormDiagnosticsRouteAuthorityAuditDigest;
void taskFormVerificationRouteAuthorityContinuityDigest;
void taskFormVerificationRouteAuthorityContinuityBehavior;
void taskEditorGraphExportDefinition;
void taskEditorGraphExportSnapshot;
void taskEditorGraphImportPosture;
void taskEditorGraphContractHistory;
void restoredTaskEditorGraph;
void restoredTaskEditorGraphHistory;
void restoredTaskEditorGraphImportPosture;
void taskEditorGraphPatchCommit;
void taskEditorGraphApplyCommit;
void taskEditorGraphTxCommit;
void authorityGraph;
void authorityGraphContract;
void authorityGraphOperationalContract;
void authorityGraphRead;
void authorityGraphInputRead;
void authorityGraphReadOnlyAuthority;
void authorityGraphImportedAuthority;
void authorityGraphPatchId;
void requirednessGraph;
void requirednessGraphDescriptors;
void requirednessServerRequiredness;
void requirednessDraftRequiredness;
void requirednessAuthorityRequiredness;
void authorityGraphWriteCommit;
void authorityGraphPatchCommit;
void authorityGraphResetCommit;
void authorityGraphApplyCommit;
void authorityGraphTransactionCommit;
void explicitCallbackPanel;
void panelSnapshotFromRead;
void countSnapshotFromRead;
void nameInput;
void scopedSignals;
void nestedScopedSignals;
void scopedDescriptor;
void scopedCanonicalCountId;
void routes;
void projectedRoutes;
void scopedRoutes;
void rawLocationAuthority;
void rawLocationNavigationType;
void canonicalUrlAuthority;
void canonicalUrlAuthorityHref;
void canonicalUrlAuthorityDigest;
void userDetailLocation;
void userDetailCanonical;
void userDetailCanonicalHref;
void userDetailCanonicalDigest;
void projectedAuthSource;
void projectedWorkspaceReadySource;
void projectedTenantCapabilitySource;
void userDetailReferenceVerification;
void userDetailRouteSchemaDigest;
void userDetailCanonicalVerification;
void userDetailCanonicalUrlDigest;
void userDetailIntent;
void userDetailIntentDescriptorKind;
void userDetailIntentCanonicalDigest;
void userDetailIntentVerificationDigest;
void userDetailPlan;
void userDetailPlanKind;
void userDetailPlanHref;
void userDetailPlanCost;
void userDetailPlanLooksExpensive;
void userDetailPlanProjectionRefresh;
void userDetailPlanCanonicalDigest;
void userDetailPlanExplainCanonicalDigest;
void userDetailPlanVerificationDigest;
void userDetailPlanExplainabilityDigest;
void directUserDetailPlan;
void directUserDetailPlanKind;
void directUserDetailPlanExplanationHref;
void directUserDetailCanonicalDigest;
void userDetailHref;
void matchedUserDetail;
void matchedUserDetailId;
void matchedUserDetailPage;
void matchedUserDetailActive;
void matchedUserDetailHash;
void matchedFromRawAuthority;
void matchedFromCanonicalAuthority;
void projectedCandidate;
void projectedRoute;
void projectedRouteId;
void projectedRouteParamId;
void projectedRouteCanonicalDigest;
void projectedAdmissionPlan;
void projectedAdmissionPlanDigest;
void projectedAdmissionRecoveryNames;
void projectedAdmissionPlanProvenanceAttemptedRouteId;
void projectedAdmissionPlanConsumedSourceName;
void projectedAdmissionOutcome;
void projectedAdmissionOutcomeKind;
void projectedAdmissionOutcomeRecovery;
void projectedAdmissionDiagnosticsRecovery;
void projectedAdmissionOutcomeProvenance;
void projectedAdmissionOutcomeTerminalSource;
void projectedAdmissionOutcomeRecoveryTrail;
void projectedAdmissionFormsAuthority;
void projectedAdmissionFormsAuthoritySurfaceId;
void projectedAdmissionFormsAuthorityDigest;
void projectedAdmissionOutcomeConsumedSourceFamily;
void projectedLayouts;
void projectedOutletId;
void projectedCandidateDigest;
void projectedAppOutletId;
void projectedUsersOutletId;
void projectedRouteAuthorityForm;
void projectedRouteAuthoritySummary;
void projectedRouteAuthorityContinuity;
void projectedRouteAuthorityContinuityApplied;
void projectedRouteAuthorityTransitionKind;
void projectedRouteAuthorityPreviousAuthorityDigest;
void projectedRouteAuthorityChangedReports;
void scopedRouteHref;
void routeLocationCheck;
void rawLocationCheck;
void canonicalUrlCheck;
void forgedCanonicalArtifact;
void forgedRawLocationAuthority;
void forgedCanonicalUrlAuthority;
void forgedRouteVerificationPackage;
void forgedRawLocationVerificationPackage;
void forgedCanonicalUrlVerificationPackage;
void forgedCanonicalVerificationPackage;
void forgedNavigationIntentVerificationPackage;
void forgedNavigationPlanVerificationPackage;
void forgedProjectedRouteCapability;
void forgedProjectedRouteCandidate;
void scopedCounterGraph;
void definitions;
void runtimeEnvelope;
void runtimeEnvelopeRestoreMode;
void transportReport;
void restoredBranchId;
void snapshotExplanationRetention;
void checkpointImage;
void diagnosticGraph;
void maybeUnavailable;
void proof;
void proofVersion;
void proofDigest;
void diagnostics;
void history;
void specialist;
void currentBranch;
void branchReplay;
void branchSnapshot;
void branchEnvelope;
void branchSnapshotRestoreMode;
void branchEnvelopeRestoreMode;
void branchProof;
void parityProof;
void artifactProof;
void graphProfile;
void specialistGraphProfile;
void specialistTouchedNodes;
void callbackNodeIds;
void latestHistoryNode;
void latestFailureMessage;
void latestInvalidationPlanningSeedCount;
void latestHostCapabilityEventKind;
void latestHostCapabilityEventQueuedCount;
void latestHostCapabilityDeniedIds;
void hostCapabilityReport;
void hostCapabilityLineageDigest;
void hostCapabilityBreadthDigest;
void hostCapabilityLineageEntry;
void hostCapabilityBreadthFamily;
void hostCapabilityInvalidationCount;
void hostCapabilityReadCount;
void hostCapabilityPollCount;
void hostCapabilityNoOpPollCount;
void hostCapabilityManualCommitCount;
void hostCapabilityNoOpManualCommitCount;
void hostCapabilityReevaluationCount;
void hostCapabilityCompatibilityDenialCount;
void hostCapabilityUnavailabilityArtifactCount;
void hostCapabilityBroadFanoutDenialCount;
void latestHostCapabilityRead;
void recentHostCapabilityEvents;
void unavailableHostCapabilityTransport;
void latestCallbackCurrentReads;
void branchReplayCallback;
void branchSnapshotBranchId;
void branchEnvelopeSnapshotId;
void parityMismatchCount;
void artifactParity;
void viewportState;
void viewportSize;
void viewportWidth;
void viewportHeight;
void viewportDescriptor;
void visibilityState;
void visibilityStateNow;
void visibilityFlag;
void visibilityDescriptor;
void onlineState;
void onlineStateNow;
void onlineFlag;
void onlineDescriptor;
void clockTick;
void clockNow;
void clockDescriptor;
void persistedDraft;
void persistenceValue;
void persistenceMode;
void persistenceRevision;
void persistenceDescriptor;
void persistenceCommit;
void forgedSignal;
await signals.terminate();
