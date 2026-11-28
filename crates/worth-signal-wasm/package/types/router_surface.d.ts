import type {
  RouteBreadcrumbContext,
  RouteCarriedBreadcrumbs,
  RouteCarriedBreadcrumbsVerificationPackage,
  RouteBreadcrumbDeclaration,
  RouteBreadcrumbEntry,
  RouteBreadcrumbEntryDeclaration,
  RouteBreadcrumbEntryDeclarationOptions,
  RouteBreadcrumbParentDeclaration,
  RouteBreadcrumbProvenance,
  RouteBreadcrumbProvenanceVerificationPackage,
  RouteBreadcrumbSourceKind,
  RouteBreadcrumbStatus,
  RouteBreadcrumbTarget,
  RouteBreadcrumbTargetKind,
  RouteBreadcrumbTrail,
  RouteBreadcrumbTrailDeclaration,
  RouteBreadcrumbEntryVerificationPackage,
  RouteBreadcrumbTrailVerificationPackage,
  RouteRestoredBreadcrumbs,
  RouteRestoredBreadcrumbsVerificationPackage,
} from "./router_breadcrumb_surface.js";
import type {
  RouteConstraint,
  RoutePathParams,
} from "./router/route_types.js";
import type {
  RouteControllerMap,
  RouteGraphMap,
  RouterRouteCompositionOptions,
} from "./router_composition_surface.js";
import type {
  AdmittedRouteResourceCapability,
  ProjectedRouteResourceCapability,
  RouteResourceCurrentState,
  RouteResourceDeclaration,
  RouteResourceDeclarationVerificationPackage,
  RouteResourceMap,
  RouteResourcePrefetchArtifact,
  RouteResourcePrefetchPosture,
  RouteResourcePrefetchVerificationPackage,
  RouteResourceResolveContext,
  RouteResourceLineFamily,
} from "./router_resource_surface.js";
import type {
  CanonicalUrlAuthority,
  CanonicalUrlVerificationPackage,
  RawLocationAuthority,
  RawLocationNavigationType,
  RawLocationOptions,
  RawLocationVerificationPackage,
  UrlSearchParamEntry,
} from "./router_authority_surface.js";
import type {
  BrowserAuthorityCoherenceKind,
  BrowserHistoryNavigationKind,
  RouterBrowserAuthorityCoherence,
  RouterBrowserAuthorityCoherenceNamespace,
  RouterBrowserAuthorityCoherenceVerificationPackage,
  RouterBrowserHistoryAdmissionReport,
  RouterBrowserHistoryAdmissionVerificationPackage,
  RouterBrowserHistoryBoundaryEvent,
  RouterBrowserHistoryBoundaryEventVerificationPackage,
  RouterBrowserHistoryBoundaryReport,
  RouterBrowserHistoryBackProvenance,
  RouterBrowserHistoryBackProvenanceVerificationPackage,
  RouterBrowserHistoryBreadcrumbTrail,
  RouterBrowserHistoryBreadcrumbTrailVerificationPackage,
  RouterBrowserHistoryIngress,
  RouterBrowserHistoryIngressOptions,
  RouterBrowserHistoryIngressVerificationPackage,
  RouterBrowserHistoryNamespace,
  RouterBrowserHistoryStory,
  RouterBrowserHistoryStoryEntry,
  RouterBrowserHistoryStoryEntryVerificationPackage,
  RouterBrowserHistoryStoryVerificationPackage,
  RouterBrowserHistoryWriteback,
  RouterBrowserHistoryLocalWritebackOptions,
  RouterBrowserHistoryWritebackNamespace,
  RouterBrowserHistoryWritebackOptions,
  RouterBrowserHistoryWritebackReport,
  RouterBrowserHistoryWritebackReportVerificationPackage,
  RouterBrowserHistoryWritebackTarget,
  RouterBrowserHistoryWritebackVerificationPackage,
} from "./router_history_surface.js";
import type {
  RouterHydrationAdmissionReport,
  RouterHydrationAdmissionVerificationPackage,
  RouterHydrationHandoff,
  RouterHydrationHandoffVerificationPackage,
  RouterHydrationNamespace,
  RouterHydrationServerOptions,
} from "./router_hydration_surface.js";
import type {
  RouteHistoryReplayResult,
  RouteHistoryReplayResultVerificationPackage,
  RouteHistoryRestoreResult,
  RouteHistoryRestoreResultVerificationPackage,
  RouteReplayHistoryFacade,
  RouteRestoreBoundary,
  RouteRestoreBoundaryGuarantees,
  RouteRestoreBoundaryVerificationPackage,
  RouteRestoreHistoryFacade,
} from "./router_restore_surface.js";
import type {
  RouterWarmupIngress,
  RouterWarmupIngressOptions,
  RouterWarmupIngressVerificationPackage,
  RouterWarmupNamespace,
  RouterWarmupReport,
  RouterWarmupReportVerificationPackage,
} from "./router_warmup_surface.js";
import type {
  NavigationArtifactPolicy,
  NavigationCommitPolicy,
  NavigationContinuityPolicy,
  NavigationDeployment,
  NavigationRedirectPolicy,
  NavigationIntentKind,
  NavigationIntentOptions,
  NavigationPolicy,
  NavigationProjectionRefreshPolicy,
} from "./router_navigation_surface.js";
import type {
  ProjectedRoutePrefetchArtifact,
  RoutePrefetchTrigger,
  RoutePrefetchVerificationPackage,
  RouteTransitionArtifact,
  RouteTransitionDiagnostics,
  RouteTransitionOptions,
  RouteTransitionRequestedSource,
  RouteTransitionTarget,
  RouteTransitionVerificationPackage,
  RouteTransitionVisiblePolicy,
  RouteVisibleChangeSource,
} from "./router_transition_surface.js";
import type {
  RouteNavigationCost,
  RouteNavigationExecutionContract,
  RouteNavigationExplanation,
  RouteNavigationFreshnessDiagnostics,
  RouteNavigationProjectionPolicy,
  RouteNavigationTransitionPolicy,
} from "./router_navigation_artifact_surface.js";
import type {
  CanonicalRouteVerificationPackage,
  NavigationIntentVerificationPackage,
  NavigationPlanVerificationPackage,
  RouteReferenceVerificationPackage,
} from "./router_verification_surface.js";
import type {
  ProjectedControllerCapability,
  ProjectedGraphCapability,
  ProjectedLayoutPlacement,
  ProjectedOutletContract,
  ProjectedOutletContractVerificationPackage,
  ProjectedOutletDescriptor,
  ProjectedOutletOccupant,
  ProjectedRouteCandidate,
  ProjectedRouteCandidateVerificationPackage,
  ProjectedRouteCapability,
  RouterLayoutDeclaration,
  RouterLayoutOptions,
} from "./router_projection_surface.js";
import type {
  AdmittedControllerCapability,
  AdmittedGraphCapability,
  AdmittedRouteCapability,
  RouteAdmissionSource,
  RouteAdmissionSourceFamily,
  RouteAdmissionSourceNamespace,
  RouteAdmissionSourceValueKind,
  RouteAdmissionDiagnostics,
  RouteAdmissionDecisionProvenance,
  RouteAdmissionFacts,
  RouteAdmissionPlan,
  RouteAdmissionPlanProvenance,
  RouteAdmissionPlanVerificationPackage,
  RouteOutcome,
  RouteOutcomeProvenance,
  RouteOutcomeVerificationPackage,
  RoutePrerequisiteDeclaration,
  RoutePrerequisiteEvaluationContext,
  RouteRecoveryProvenance,
  RouteRecoveryArtifact,
  RouteRecoveryDeclaration,
  RouteRecoveryEvaluationContext,
  RouteFormsAuthorityArtifact,
  RouteFormsAuthorityContinuity,
  RouteFormsAuthorityDeclaration,
  RouteFormsAuthorityVerificationPackage,
} from "./router_admission_surface.js";
import type {
  RouterDefinitionTree,
  RouterResolvedTree,
} from "./router_tree_surface.js";

export type {
  RouteBreadcrumbContext,
  RouteCarriedBreadcrumbs,
  RouteCarriedBreadcrumbsVerificationPackage,
  RouteBreadcrumbDeclaration,
  RouteBreadcrumbEntry,
  RouteBreadcrumbEntryDeclaration,
  RouteBreadcrumbEntryDeclarationOptions,
  RouteBreadcrumbEntryVerificationPackage,
  RouteBreadcrumbParentDeclaration,
  RouteBreadcrumbProvenance,
  RouteBreadcrumbProvenanceVerificationPackage,
  RouteBreadcrumbSourceKind,
  RouteBreadcrumbStatus,
  RouteBreadcrumbTarget,
  RouteBreadcrumbTargetKind,
  RouteBreadcrumbTrail,
  RouteBreadcrumbTrailDeclaration,
  RouteBreadcrumbTrailVerificationPackage,
  RouteRestoredBreadcrumbs,
  RouteRestoredBreadcrumbsVerificationPackage,
} from "./router_breadcrumb_surface.js";
export type {
  RouteControllerMap,
  RouteGraphMap,
  RouterRouteCompositionOptions,
} from "./router_composition_surface.js";
export type {
  AdmittedRouteResourceCapability,
  ProjectedRouteResourceCapability,
  RouteResourceCurrentState,
  RouteResourceDeclaration,
  RouteResourceDeclarationVerificationPackage,
  RouteResourceMap,
  RouteResourcePrefetchArtifact,
  RouteResourcePrefetchPosture,
  RouteResourcePrefetchVerificationPackage,
  RouteResourceResolveContext,
  RouteResourceLineFamily,
} from "./router_resource_surface.js";
export type {
  ProjectedControllerCapability,
  ProjectedGraphCapability,
  ProjectedLayoutPlacement,
  ProjectedOutletContract,
  ProjectedOutletContractVerificationPackage,
  ProjectedOutletDescriptor,
  ProjectedOutletOccupant,
  ProjectedRouteCandidate,
  ProjectedRouteCandidateVerificationPackage,
  ProjectedRouteCapability,
  RouterLayoutDeclaration,
  RouterLayoutOptions,
} from "./router_projection_surface.js";
export type {
  AdmittedControllerCapability,
  AdmittedGraphCapability,
  RouteAdmissionDiagnostics,
  RouteAdmissionDecisionProvenance,
  RouteAdmissionSource,
  RouteAdmissionSourceFamily,
  RouteAdmissionSourceNamespace,
  RouteAdmissionSourceValueKind,
  AdmittedRouteCapability,
  RouteAdmissionFacts,
  RouteAdmissionPlan,
  RouteAdmissionPlanProvenance,
  RouteAdmissionPlanVerificationPackage,
  RouteOutcome,
  RouteOutcomeProvenance,
  RouteOutcomeVerificationPackage,
  RoutePrerequisiteDeclaration,
  RoutePrerequisiteEvaluationContext,
  RouteRecoveryProvenance,
  RouteRecoveryArtifact,
  RouteRecoveryDeclaration,
  RouteRecoveryEvaluationContext,
  RouteFormsAuthorityArtifact,
  RouteFormsAuthorityContinuity,
  RouteFormsAuthorityDeclaration,
  RouteFormsAuthorityVerificationPackage,
} from "./router_admission_surface.js";
export type {
  RouterDefinitionTree,
  RouterResolvedTree,
} from "./router_tree_surface.js";
export type {
  CanonicalUrlAuthority,
  CanonicalUrlVerificationPackage,
  RawLocationAuthority,
  RawLocationNavigationType,
  RawLocationOptions,
  RawLocationVerificationPackage,
  UrlSearchParamEntry,
} from "./router_authority_surface.js";
export type {
  BrowserAuthorityCoherenceKind,
  BrowserHistoryNavigationKind,
  RouterBrowserAuthorityCoherence,
  RouterBrowserAuthorityCoherenceNamespace,
  RouterBrowserAuthorityCoherenceVerificationPackage,
  RouterBrowserHistoryAdmissionReport,
  RouterBrowserHistoryAdmissionVerificationPackage,
  RouterBrowserHistoryBoundaryEvent,
  RouterBrowserHistoryBoundaryEventVerificationPackage,
  RouterBrowserHistoryBoundaryReport,
  RouterBrowserHistoryBackProvenance,
  RouterBrowserHistoryBackProvenanceVerificationPackage,
  RouterBrowserHistoryBreadcrumbTrail,
  RouterBrowserHistoryBreadcrumbTrailVerificationPackage,
  RouterBrowserHistoryIngress,
  RouterBrowserHistoryIngressOptions,
  RouterBrowserHistoryIngressVerificationPackage,
  RouterBrowserHistoryNamespace,
  RouterBrowserHistoryStory,
  RouterBrowserHistoryStoryEntry,
  RouterBrowserHistoryStoryEntryVerificationPackage,
  RouterBrowserHistoryStoryVerificationPackage,
  RouterBrowserHistoryWriteback,
  RouterBrowserHistoryLocalWritebackOptions,
  RouterBrowserHistoryWritebackNamespace,
  RouterBrowserHistoryWritebackOptions,
  RouterBrowserHistoryWritebackReport,
  RouterBrowserHistoryWritebackReportVerificationPackage,
  RouterBrowserHistoryWritebackTarget,
  RouterBrowserHistoryWritebackVerificationPackage,
} from "./router_history_surface.js";
export type {
  RouterHydrationAdmissionReport,
  RouterHydrationAdmissionVerificationPackage,
  RouterHydrationHandoff,
  RouterHydrationHandoffVerificationPackage,
  RouterHydrationNamespace,
  RouterHydrationServerOptions,
} from "./router_hydration_surface.js";
export type {
  RouteHistoryReplayResult,
  RouteHistoryReplayResultVerificationPackage,
  RouteHistoryRestoreResult,
  RouteHistoryRestoreResultVerificationPackage,
  RouteReplayHistoryFacade,
  RouteRestoreBoundary,
  RouteRestoreBoundaryGuarantees,
  RouteRestoreBoundaryVerificationPackage,
  RouteRestoreHistoryFacade,
} from "./router_restore_surface.js";
export type {
  NavigationArtifactPolicy,
  NavigationCommitPolicy,
  NavigationContinuityPolicy,
  NavigationDeployment,
  NavigationRedirectPolicy,
  NavigationIntentKind,
  NavigationIntentOptions,
  NavigationPolicy,
  NavigationProjectionRefreshPolicy,
} from "./router_navigation_surface.js";
export type {
  ProjectedRoutePrefetchArtifact,
  RoutePrefetchTrigger,
  RoutePrefetchVerificationPackage,
  RouteTransitionArtifact,
  RouteTransitionDiagnostics,
  RouteTransitionOptions,
  RouteTransitionRequestedSource,
  RouteTransitionTarget,
  RouteTransitionVerificationPackage,
  RouteTransitionVisiblePolicy,
  RouteVisibleChangeSource,
} from "./router_transition_surface.js";
export type {
  RouteNavigationCost,
  RouteNavigationExecutionContract,
  RouteNavigationExplanation,
  RouteNavigationFreshnessDiagnostics,
  RouteNavigationProjectionPolicy,
  RouteNavigationTransitionPolicy,
} from "./router_navigation_artifact_surface.js";
export type {
  CanonicalRouteVerificationPackage,
  NavigationIntentVerificationPackage,
  NavigationPlanVerificationPackage,
  RouteReferenceVerificationPackage,
} from "./router_verification_surface.js";

declare const WorthSignalRouteDeclarationBrand: unique symbol;
declare const WorthSignalRouteLayoutReferenceBrand: unique symbol;
declare const WorthSignalRouteReferenceBrand: unique symbol;
declare const WorthSignalRouteLocationBrand: unique symbol;
declare const WorthSignalRouteNavigationIntentBuilderBrand: unique symbol;
declare const WorthSignalRouteNavigationPlanBrand: unique symbol;
declare const WorthSignalCanonicalRouteArtifactBrand: unique symbol;

export interface RouterSearchField<TValue, TRequired extends boolean> {
  readonly family: "routerSearchField";
  readonly valueKind: "string" | "number" | "boolean";
  readonly required: TRequired;
  readonly __valueType?: TValue;
}

export interface RouterHashField<TValue> {
  readonly family: "routerHashField";
  readonly valueKind: "string";
  readonly __valueType?: TValue;
}

export interface RouterSearchNamespace {
  readonly optional: {
    string(): RouterSearchField<string, false>;
    number(): RouterSearchField<number, false>;
    boolean(): RouterSearchField<boolean, false>;
  };
  readonly required: {
    string(): RouterSearchField<string, true>;
    number(): RouterSearchField<number, true>;
    boolean(): RouterSearchField<boolean, true>;
  };
}

export interface RouterHashNamespace {
  string(): RouterHashField<string>;
}

export type RouterSearchSchema = Record<string, RouterSearchField<unknown, boolean>>;

export interface RouterRouteOptions<
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
  TControllers extends RouteControllerMap = Record<string, never>,
  TGraphs extends RouteGraphMap = Record<string, never>,
> extends RouterRouteCompositionOptions<TControllers, TGraphs, RouteResourceMap> {
  search?: TSearch;
  hash?: THash;
  breadcrumb?: RouteBreadcrumbDeclaration;
  admission?: ReadonlyArray<RoutePrerequisiteDeclaration>;
  recovery?: ReadonlyArray<RouteRecoveryDeclaration>;
  forms?: RouteFormsAuthorityDeclaration;
}

export interface RouterRouteDeclaration<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
  TControllers extends RouteControllerMap = Record<string, never>,
  TGraphs extends RouteGraphMap = Record<string, never>,
> {
  readonly [WorthSignalRouteDeclarationBrand]: "routeDeclaration";
  readonly route: TRoute;
  readonly search: TSearch;
  readonly hash: THash;
  readonly controllers: TControllers;
  readonly graphs: TGraphs;
  readonly resources: RouteResourceMap;
  readonly breadcrumb: RouteBreadcrumbDeclaration | null;
  readonly admission: ReadonlyArray<RoutePrerequisiteDeclaration>;
  readonly recovery: ReadonlyArray<RouteRecoveryDeclaration>;
  readonly forms: RouteFormsAuthorityDeclaration | null;
}

type SearchFieldInput<TField> =
  TField extends RouterSearchField<infer TValue, boolean> ? TValue : never;

type RequiredSearchKeys<TSearch extends RouterSearchSchema> = {
  [K in keyof TSearch]:
    TSearch[K] extends RouterSearchField<unknown, true> ? K : never;
}[keyof TSearch];

type OptionalSearchKeys<TSearch extends RouterSearchSchema> = Exclude<
  keyof TSearch,
  RequiredSearchKeys<TSearch>
>;

export type RouterSearchInput<TSearch extends RouterSearchSchema> =
  { [K in RequiredSearchKeys<TSearch>]: SearchFieldInput<TSearch[K]> } &
  { [K in OptionalSearchKeys<TSearch>]?: SearchFieldInput<TSearch[K]> };

export type RouterSearchMatch<TSearch extends RouterSearchSchema> =
  { [K in RequiredSearchKeys<TSearch>]: SearchFieldInput<TSearch[K]> } &
  { [K in OptionalSearchKeys<TSearch>]?: SearchFieldInput<TSearch[K]> | undefined };

export type RouterHashInput<THash> =
  THash extends RouterHashField<infer TValue> ? TValue : never;

export interface RouterRouteInput<
  TRoute extends string,
  TSearch extends RouterSearchSchema,
  THash extends RouterHashField<unknown> | null,
> {
  params?: RoutePathParams<TRoute>;
  search?: RouterSearchInput<TSearch>;
  hash?: THash extends RouterHashField<unknown> ? RouterHashInput<THash> : never;
}

export interface RouterDescriptor<
  TRoute extends string = string,
  THash extends RouterHashField<unknown> | null = null,
> {
  readonly routeId: string;
  readonly scopeId: string | null;
  readonly declarationPath: ReadonlyArray<string>;
  readonly route: TRoute;
  readonly pathParamNames: ReadonlyArray<string>;
  readonly searchKeys: ReadonlyArray<string>;
  readonly hash: THash;
}

export interface CanonicalRouteArtifact<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
> {
  readonly routeId: string;
  readonly href: string;
  readonly pathname: string;
  readonly search: RouterSearchMatch<TSearch>;
  readonly hash: THash extends RouterHashField<unknown> ? RouterHashInput<THash> | undefined : undefined;
  readonly searchDigest: string;
  readonly hashDigest: string;
  readonly canonicalUrlDigest: string;
  readonly equivalenceDigest: string;
  descriptor(): RouterDescriptor<TRoute, THash>;
  verification(): CanonicalRouteVerificationPackage;
  readonly [WorthSignalCanonicalRouteArtifactBrand]: "canonicalRouteArtifact";
}

export interface RouteLocation<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
> {
  readonly route: RouteReference<TRoute, TSearch, THash>;
  readonly routeId: string;
  readonly params: RoutePathParams<TRoute>;
  readonly search: RouterSearchMatch<TSearch>;
  readonly hash: THash extends RouterHashField<unknown> ? RouterHashInput<THash> | undefined : undefined;
  readonly href: string;
  descriptor(): RouterDescriptor<TRoute, THash>;
  canonical(): CanonicalRouteArtifact<TRoute, TSearch, THash>;
  intent(options?: NavigationIntentOptions): RouteNavigationIntentBuilder<TRoute, TSearch, THash>;
  plan(policy?: NavigationPolicy): RouteNavigationPlan<TRoute, TSearch, THash>;
  readonly [WorthSignalRouteLocationBrand]: "routeLocation";
}

export interface RouteNavigationPlan<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
> {
  readonly kind: NavigationIntentKind;
  readonly routeId: string;
  readonly href: string;
  readonly params: RoutePathParams<TRoute>;
  readonly search: RouterSearchMatch<TSearch>;
  readonly hash: THash extends RouterHashField<unknown> ? RouterHashInput<THash> | undefined : undefined;
  descriptor(): RouterDescriptor<TRoute, THash>;
  canonical(): CanonicalRouteArtifact<TRoute, TSearch, THash>;
  cost(): RouteNavigationCost;
  policy(): RouteNavigationTransitionPolicy;
  execution(): RouteNavigationExecutionContract;
  explain(): RouteNavigationExplanation;
  freshness(): RouteNavigationFreshnessDiagnostics;
  verification(): NavigationPlanVerificationPackage;
  projectionPolicy(): RouteNavigationProjectionPolicy;
  readonly [WorthSignalRouteNavigationPlanBrand]: "routeNavigationPlan";
}

export interface RouteNavigationIntentDescriptor<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
> {
  readonly kind: NavigationIntentKind;
  readonly routeId: string;
  readonly href: string;
  readonly params: RoutePathParams<TRoute>;
  readonly search: RouterSearchMatch<TSearch>;
  readonly hash: THash extends RouterHashField<unknown> ? RouterHashInput<THash> | undefined : undefined;
  canonical(): CanonicalRouteArtifact<TRoute, TSearch, THash>;
}

export interface RouteNavigationIntentBuilder<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
> {
  descriptor(): RouteNavigationIntentDescriptor<TRoute, TSearch, THash>;
  verification(): NavigationIntentVerificationPackage;
  policy(policy: NavigationPolicy): RouteNavigationIntentBuilder<TRoute, TSearch, THash>;
  compile(): RouteNavigationPlan<TRoute, TSearch, THash>;
  readonly [WorthSignalRouteNavigationIntentBuilderBrand]: "routeNavigationIntentBuilder";
}

export interface RouteReference<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
  TControllers extends RouteControllerMap = Record<string, never>,
  TGraphs extends RouteGraphMap = Record<string, never>,
> {
  canonical(input?: RouterRouteInput<TRoute, TSearch, THash>): CanonicalRouteArtifact<TRoute, TSearch, THash>;
  href(input?: RouterRouteInput<TRoute, TSearch, THash>): string;
  to(input?: RouterRouteInput<TRoute, TSearch, THash>): RouteLocation<TRoute, TSearch, THash>;
  intent(
    input?: RouterRouteInput<TRoute, TSearch, THash>,
    options?: NavigationIntentOptions,
  ): RouteNavigationIntentBuilder<TRoute, TSearch, THash>;
  match(
    rawHref: string | RawLocationAuthority | CanonicalUrlAuthority,
  ): RouteLocation<TRoute, TSearch, THash> | null;
  descriptor(): RouterDescriptor<TRoute, THash>;
  verification(): RouteReferenceVerificationPackage;
  readonly [WorthSignalRouteReferenceBrand]: "routeReference";
}

export interface RouteLayoutReference<
  TRoute extends string = string,
  TSearch extends RouterSearchSchema = Record<string, never>,
  THash extends RouterHashField<unknown> | null = null,
  TControllers extends RouteControllerMap = Record<string, never>,
  TGraphs extends RouteGraphMap = Record<string, never>,
> extends RouteReference<TRoute, TSearch, THash, TControllers, TGraphs> {
  readonly outletId: string;
  readonly [WorthSignalRouteLayoutReferenceBrand]: "routeLayoutReference";
}

export interface RouterNamespace {
  readonly search: RouterSearchNamespace;
  readonly hash: RouterHashNamespace;
  readonly browserHistory: RouterBrowserHistoryNamespace;
  readonly hydration: RouterHydrationNamespace;
  readonly warmup: RouterWarmupNamespace;
  readonly host: RouteAdmissionSourceNamespace<"hostCapability">;
  readonly resource: RouteAdmissionSourceNamespace<"resourceTruth">;
  readonly graph: RouteAdmissionSourceNamespace<"graphTruth">;
  raw(href: string, options?: RawLocationOptions): RawLocationAuthority;
  canonical(href: string, options?: RawLocationOptions): CanonicalUrlAuthority;
  resourceLine<TFamily extends RouteResourceLineFamily>(
    family: TFamily,
    options: {
      params: (
        route: RouteResourceResolveContext,
      ) => Parameters<TFamily["line"]>[0];
      prefetch?: RouteResourcePrefetchPosture;
    },
  ): RouteResourceDeclaration<TFamily>;
  route<
    const TRoute extends string,
    TSearch extends RouterSearchSchema = Record<string, never>,
    THash extends RouterHashField<unknown> | null = null,
    TControllers extends RouteControllerMap = Record<string, never>,
    TGraphs extends RouteGraphMap = Record<string, never>,
  >(
    route: TRoute & RouteConstraint<TRoute>,
    options?: RouterRouteOptions<TSearch, THash, TControllers, TGraphs>,
  ): RouterRouteDeclaration<TRoute, TSearch, THash, TControllers, TGraphs>;
  forms(
    surfaceId: string,
    options?: {
      readonly continuity?: RouteFormsAuthorityContinuity;
      readonly reason?: string;
    },
  ): RouteFormsAuthorityDeclaration;
  breadcrumb(options: {
    readonly id: string;
    readonly label: string | ((context: RouteBreadcrumbContext<any, any, any>) => string);
    readonly target?:
      | RouteBreadcrumbTarget
      | ((context: RouteBreadcrumbContext<any, any, any>) => RouteBreadcrumbTarget | null | undefined)
      | null;
    readonly parent?: RouteBreadcrumbParentDeclaration;
  }): RouteBreadcrumbDeclaration;
  breadcrumbEntry(
    options: RouteBreadcrumbEntryDeclarationOptions,
  ): RouteBreadcrumbEntryDeclaration;
  breadcrumbParent(options: {
    readonly recompute?: (
      context: RouteBreadcrumbContext<any, any, any>,
    ) => RouteBreadcrumbEntryDeclaration | RouteBreadcrumbTrailDeclaration | null | undefined;
    readonly carry?: boolean;
    readonly fallback?: RouteBreadcrumbEntryDeclaration | RouteBreadcrumbTrailDeclaration;
  }): RouteBreadcrumbParentDeclaration;
  breadcrumbTrail(
    entries: ReadonlyArray<RouteBreadcrumbEntryDeclaration>,
  ): RouteBreadcrumbTrailDeclaration;
  carryBreadcrumbs(
    trail: RouteBreadcrumbTrail | ReadonlyArray<RouteBreadcrumbEntry>,
  ): RouteCarriedBreadcrumbs;
  restoreBreadcrumbs(
    trail: RouteBreadcrumbTrail | ReadonlyArray<RouteBreadcrumbEntry>,
  ): RouteRestoredBreadcrumbs;
  restoreBoundary(
    snapshotEnvelopeArtifact: import("./callable_surface.js").RuntimeSnapshotEnvelopeArtifact,
  ): RouteRestoreBoundary;
  prerequisite(
    name: string,
    evaluate: (
      context: RoutePrerequisiteEvaluationContext<string, RouterSearchSchema, RouterHashField<unknown> | null>,
    ) => unknown | Promise<unknown>,
  ): RoutePrerequisiteDeclaration;
  prerequisite<const TConsumedSources extends ReadonlyArray<RouteAdmissionSource>>(
    name: string,
    options: {
      consumes: TConsumedSources;
      evaluate: (
        context: RoutePrerequisiteEvaluationContext<
          string,
          RouterSearchSchema,
          RouterHashField<unknown> | null,
          TConsumedSources
        >,
      ) => unknown | Promise<unknown>;
    },
  ): RoutePrerequisiteDeclaration<TConsumedSources>;
  recovery(
    name: string,
    evaluate: (
      context: RouteRecoveryEvaluationContext<string, RouterSearchSchema, RouterHashField<unknown> | null>,
    ) => unknown | Promise<unknown>,
  ): RouteRecoveryDeclaration;
  layout<
    const TRoute extends string,
    TChildren extends RouterDefinitionTree,
  >(
    route: TRoute & RouteConstraint<TRoute>,
    children: TChildren,
  ): RouterLayoutDeclaration<TRoute, Record<string, never>, null, Record<string, never>, Record<string, never>, TChildren>;
  layout<
    const TRoute extends string,
    TChildren extends RouterDefinitionTree,
  >(
    route: TRoute & RouteConstraint<TRoute>,
    options: RouterLayoutOptions,
    children: TChildren,
  ): RouterLayoutDeclaration<TRoute, Record<string, never>, null, Record<string, never>, Record<string, never>, TChildren>;
  layout<
    TRoute extends string,
    TSearch extends RouterSearchSchema,
    THash extends RouterHashField<unknown> | null,
    TControllers extends RouteControllerMap,
    TGraphs extends RouteGraphMap,
    TChildren extends RouterDefinitionTree,
  >(
    routeDeclaration: RouterRouteDeclaration<TRoute, TSearch, THash, TControllers, TGraphs>,
    children: TChildren,
  ): RouterLayoutDeclaration<TRoute, TSearch, THash, TControllers, TGraphs, TChildren>;
  layout<
    TRoute extends string,
    TSearch extends RouterSearchSchema,
    THash extends RouterHashField<unknown> | null,
    TControllers extends RouteControllerMap,
    TGraphs extends RouteGraphMap,
    TChildren extends RouterDefinitionTree,
  >(
    routeDeclaration: RouterRouteDeclaration<TRoute, TSearch, THash, TControllers, TGraphs>,
    options: RouterLayoutOptions,
    children: TChildren,
  ): RouterLayoutDeclaration<TRoute, TSearch, THash, TControllers, TGraphs, TChildren>;
  define<const TTree extends RouterDefinitionTree>(definitions: TTree): RouterResolvedTree<TTree>;
  isRouteLocation(value: unknown): value is RouteLocation;
  isRawLocationAuthority(value: unknown): value is RawLocationAuthority;
  isCanonicalUrlAuthority(value: unknown): value is CanonicalUrlAuthority;
}

export type {
  RouteConstraint,
  RoutePathParams,
} from "./router/route_types.js";
