import type {
  CanonicalUrlAuthority,
  RawLocationAuthority,
} from "./router_authority_surface.js";
import type {
  RouterBrowserHistoryAdmissionReport,
  RouterBrowserHistoryIngress,
  RouterBrowserHistoryWriteback,
  RouterBrowserHistoryWritebackReport,
} from "./router_history_surface.js";
import type {
  RouterHydrationAdmissionReport,
  RouterHydrationHandoff,
} from "./router_hydration_surface.js";
import type {
  AdmittedRouteCapability,
  RouteAdmissionFacts,
  RouteOutcome,
} from "./router_admission_surface.js";
import type {
  RouteControllerMap,
  RouteGraphMap,
} from "./router_composition_surface.js";
import type {
  ProjectedLayoutPlacement,
  ProjectedRouteCandidate,
  ProjectedRouteCapability,
  RouterLayoutDeclaration,
} from "./router_projection_surface.js";
import type {
  RouteTransitionArtifact,
  RouteTransitionOptions,
  RouteTransitionTarget,
} from "./router_transition_surface.js";
import type {
  RouterSequenceNavigationStep,
  RouterSequenceScenario,
} from "./router_sequence_surface.js";
import type {
  RouterWarmupIngress,
  RouterWarmupReport,
} from "./router_warmup_surface.js";
import type {
  RouteLayoutReference,
  RouteLocation,
  RouteReference,
  RouterHashField,
  RouterRouteDeclaration,
  RouterSearchSchema,
} from "./router_surface.js";

export type RouterDefinitionTree = {
  readonly [key: string]:
    | RouterRouteDeclaration<
      string,
      RouterSearchSchema,
      RouterHashField<unknown> | null,
      RouteControllerMap,
      RouteGraphMap
    >
    | RouterLayoutDeclaration<
      string,
      RouterSearchSchema,
      RouterHashField<unknown> | null,
      RouteControllerMap,
      RouteGraphMap,
      RouterDefinitionTree
    >
    | RouterDefinitionTree;
};

type RouterResolvedLeafUnion<TTree> = {
  readonly [K in keyof TTree]:
    TTree[K] extends RouterRouteDeclaration<
      infer TRoute extends string,
      infer TSearch extends RouterSearchSchema,
      infer THash extends RouterHashField<unknown> | null,
      infer TControllers extends RouteControllerMap,
      infer TGraphs extends RouteGraphMap
    >
      ? ProjectedRouteCapability<TRoute, TSearch, THash, TControllers, TGraphs>
      : TTree[K] extends RouterLayoutDeclaration<any, any, any, any, any, infer TChildren extends RouterDefinitionTree>
        ? RouterResolvedLeafUnion<TChildren>
        : TTree[K] extends RouterDefinitionTree
          ? RouterResolvedLeafUnion<TTree[K]>
          : never;
}[keyof TTree];

type RouterResolvedLayoutUnion<TTree> = {
  readonly [K in keyof TTree]:
    TTree[K] extends RouterLayoutDeclaration<
      infer TRoute extends string,
      infer TSearch extends RouterSearchSchema,
      infer THash extends RouterHashField<unknown> | null,
      infer TControllers extends RouteControllerMap,
      infer TGraphs extends RouteGraphMap,
      infer TChildren extends RouterDefinitionTree
    >
      ? ProjectedLayoutPlacement<TRoute, TSearch, THash, TControllers, TGraphs> | RouterResolvedLayoutUnion<TChildren>
      : TTree[K] extends RouterDefinitionTree
        ? RouterResolvedLayoutUnion<TTree[K]>
        : never;
}[keyof TTree];

type RouterResolvedAdmittedLeafUnion<TTree> = {
  readonly [K in keyof TTree]:
    TTree[K] extends RouterRouteDeclaration<
      infer TRoute extends string,
      infer TSearch extends RouterSearchSchema,
      infer THash extends RouterHashField<unknown> | null,
      infer TControllers extends RouteControllerMap,
      infer TGraphs extends RouteGraphMap
    >
      ? AdmittedRouteCapability<TRoute, TSearch, THash, TControllers, TGraphs>
      : TTree[K] extends RouterLayoutDeclaration<any, any, any, any, any, infer TChildren extends RouterDefinitionTree>
        ? RouterResolvedAdmittedLeafUnion<TChildren>
        : TTree[K] extends RouterDefinitionTree
          ? RouterResolvedAdmittedLeafUnion<TTree[K]>
          : never;
}[keyof TTree];

type RouterResolvedNode<TNode> =
  TNode extends RouterRouteDeclaration<
    infer TRoute extends string,
    infer TSearch extends RouterSearchSchema,
    infer THash extends RouterHashField<unknown> | null,
    infer TControllers extends RouteControllerMap,
    infer TGraphs extends RouteGraphMap
  >
    ? RouteReference<TRoute, TSearch, THash, TControllers, TGraphs>
    : TNode extends RouterLayoutDeclaration<
      infer TRoute extends string,
      infer TSearch extends RouterSearchSchema,
      infer THash extends RouterHashField<unknown> | null,
      infer TControllers extends RouteControllerMap,
      infer TGraphs extends RouteGraphMap,
      infer TChildren extends RouterDefinitionTree
    >
      ? RouteLayoutReference<TRoute, TSearch, THash, TControllers, TGraphs> & RouterResolvedChildren<TChildren>
      : TNode extends RouterDefinitionTree
        ? RouterResolvedChildren<TNode>
        : never;

type RouterResolvedChildren<TTree> = {
  readonly [K in keyof TTree]:
    RouterResolvedNode<TTree[K]>;
};

export type RouterResolvedTree<TTree extends RouterDefinitionTree> =
  RouterResolvedChildren<TTree> & {
    project(
      rawHref: string | RawLocationAuthority | CanonicalUrlAuthority,
    ): ProjectedRouteCandidate<
      RouterResolvedLeafUnion<TTree>,
      RouterResolvedLayoutUnion<TTree>
    > | null;
    warmup(
      rawHref: string | RawLocationAuthority | CanonicalUrlAuthority,
      trigger?: import("./router_transition_surface.js").RoutePrefetchTrigger,
    ): import("./router_transition_surface.js").ProjectedRoutePrefetchArtifact | null;
    applyWarmupIngress(
      ingress: RouterWarmupIngress,
    ): RouterWarmupReport;
    admit(
      rawHref: string | RawLocationAuthority | CanonicalUrlAuthority,
      facts?: RouteAdmissionFacts,
    ): Promise<RouteOutcome<
      RouterResolvedAdmittedLeafUnion<TTree>,
      RouterResolvedLayoutUnion<TTree>
    >>;
    transition(
      currentOutcome: Extract<RouteOutcome<
        RouterResolvedAdmittedLeafUnion<TTree>,
        RouterResolvedLayoutUnion<TTree>
      >, { kind: "admitted" }>,
      target: RouteTransitionTarget,
      options?: RouteTransitionOptions,
    ): Promise<RouteTransitionArtifact>;
    admitBrowserHistoryIngress(
      ingress: RouterBrowserHistoryIngress,
      facts?: RouteAdmissionFacts,
    ): Promise<RouterBrowserHistoryAdmissionReport<RouteOutcome<
      RouterResolvedAdmittedLeafUnion<TTree>,
      RouterResolvedLayoutUnion<TTree>
    >>>;
    admitHydrationHandoff(
      handoff: RouterHydrationHandoff,
      facts?: RouteAdmissionFacts,
    ): Promise<RouterHydrationAdmissionReport<RouteOutcome<
      RouterResolvedAdmittedLeafUnion<TTree>,
      RouterResolvedLayoutUnion<TTree>
    >>>;
    applyBrowserHistoryWriteback(
      writeback: RouterBrowserHistoryWriteback,
      facts?: RouteAdmissionFacts,
    ): Promise<RouterBrowserHistoryWritebackReport<RouteOutcome<
      RouterResolvedAdmittedLeafUnion<TTree>,
      RouterResolvedLayoutUnion<TTree>
    > | null>>;
    simulateSequence(
      sequence: ReadonlyArray<RouterSequenceNavigationStep<RouteAdmissionFacts> | string | RawLocationAuthority | RouteLocation<any, any, any>>,
    ): RouterSequenceScenario<
      RouteOutcome<RouterResolvedAdmittedLeafUnion<TTree>, RouterResolvedLayoutUnion<TTree>>,
      RouteAdmissionFacts
    >;
  };
