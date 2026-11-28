import {
  createBrowserHistoryAdmissionReport,
  requireRouterBrowserHistoryIngress,
} from "./ingress/router_browser_history_ingress.js";
import {
  createRouteWarmupReport,
  requireRouterWarmupIngress,
} from "./ingress/router_warmup_ingress.js";
import {
  createBrowserHistoryWritebackReport,
  requireRouterBrowserHistoryWriteback,
} from "./ingress/router_browser_history_writeback.js";
import {
  createHydrationAdmissionReport,
  requireRouterHydrationHandoff,
} from "./ingress/router_hydration_handoff.js";
import {
  matchRoutePathPrefix,
} from "../../route/route_pattern.js";
import {
  createNotFoundRouteOutcome,
  createProjectedRouteAdmissionPlan,
} from "./admission/router_admission_resolution.js";
import {
  createProjectedLayoutPlacement,
  createProjectedRouteCapability,
} from "./router_projection_capability.js";
import {
  finalizeProjectedOutletContracts,
} from "./router_projection_outlet_contract.js";
import {
  createProjectedRoutePrefetchArtifact,
  tryCreateProjectedRouteWarmupArtifact,
} from "./transition/router_warmup_artifact.js";
import {
  createRouteTransitionArtifact,
} from "./transition/router_transition_artifact.js";
import {
  createProjectedCandidateVerification,
} from "./router_projection_verification.js";
import {
  createRouteSequenceScenario,
} from "./router_sequence_scenario.js";
import {
  ROUTE_LAYOUT_REFERENCE,
  ROUTE_PROJECTED_CANDIDATE,
  ROUTE_TREE_ROOT,
} from "../router_symbols.js";
import {
  createCanonicalUrlAuthority,
  isCanonicalUrlAuthority,
  isRawLocationAuthority,
} from "../url_authority/router_url_authority.js";

function attachProjectionRoot(tree, rootNodes, navigationSupport) {
  function projectRouteCandidate(routeAuthority) {
    return projectResolvedRouteTree(rootNodes, routeAuthority, projectRouteCandidate);
  }
  return Object.freeze({
    ...tree,
    [ROUTE_TREE_ROOT]: true,
    project(routeAuthority) {
      return projectRouteCandidate(routeAuthority);
    },
    warmup(routeAuthority, trigger = "intent") {
      const projectedCandidate = projectRouteCandidate(routeAuthority);
      if (projectedCandidate === null) {
        return null;
      }
      return projectedCandidate.warmup(trigger);
    },
    applyWarmupIngress(ingress) {
      const normalizedIngress = requireRouterWarmupIngress(
        ingress,
        "routes.applyWarmupIngress(...)",
      );
      const projectedCandidate = projectRouteCandidate(normalizedIngress.rawLocation);
      if (projectedCandidate === null) {
        return createRouteWarmupReport(normalizedIngress, null, "noProjectedCandidate");
      }
      const artifact = tryCreateProjectedRouteWarmupArtifact(
        projectedCandidate,
        normalizedIngress.trigger,
      );
      return createRouteWarmupReport(
        normalizedIngress,
        artifact,
        artifact === null ? "noMatchingWarmupResources" : "routeWarmupStarted",
      );
    },
    async admit(routeAuthority, facts = {}) {
      const projectedCandidate = projectRouteCandidate(routeAuthority);
      if (projectedCandidate === null) {
        return createNotFoundRouteOutcome(normalizeProjectionAuthority(routeAuthority), facts);
      }
      return projectedCandidate.admission(facts).resolve();
    },
    async transition(currentOutcome, target, options = {}) {
      return createRouteTransitionArtifact(this, currentOutcome, target, options);
    },
    async admitBrowserHistoryIngress(ingress, facts = {}) {
      const normalizedIngress = requireRouterBrowserHistoryIngress(
        ingress,
        "routes.admitBrowserHistoryIngress(...)",
      );
      const outcome = await this.admit(normalizedIngress.rawLocation, facts);
      return createBrowserHistoryAdmissionReport(normalizedIngress, outcome);
    },
    async admitHydrationHandoff(handoff, facts = {}) {
      const normalizedHandoff = requireRouterHydrationHandoff(
        handoff,
        "routes.admitHydrationHandoff(...)",
      );
      const outcome = await this.admit(normalizedHandoff.rawLocation, facts);
      return createHydrationAdmissionReport(normalizedHandoff, outcome);
    },
    async applyBrowserHistoryWriteback(writeback, facts = {}) {
      const normalizedWriteback = requireRouterBrowserHistoryWriteback(
        writeback,
        "routes.applyBrowserHistoryWriteback(...)",
      );
      const outcome = normalizedWriteback.rawLocation === null
        ? null
        : await this.admit(normalizedWriteback.rawLocation, facts);
      return createBrowserHistoryWritebackReport(normalizedWriteback, outcome);
    },
    simulateSequence(sequence) {
      return createRouteSequenceScenario(this, navigationSupport, sequence);
    },
  });
}

function createRouteLayoutReference(routeReference, outletId, children) {
  return Object.freeze({
    ...children,
    ...routeReference,
    outletId,
    [ROUTE_LAYOUT_REFERENCE]: true,
  });
}

function projectResolvedRouteTree(rootNodes, routeAuthority, projectRouteCandidate) {
  const canonicalUrlAuthority = normalizeProjectionAuthority(routeAuthority);
  if (canonicalUrlAuthority === null) {
    return null;
  }
  const projectedMatch = selectProjectedMatch(
    rootNodes,
    canonicalUrlAuthority,
    [],
    null,
    [],
  );
  if (projectedMatch === null) {
    return null;
  }
  const projectedRouteCapability = createProjectedRouteCapability(
    projectedMatch.declaration,
    projectedMatch.leafLocation,
  );
  const { layoutPlacements, outletContracts } = finalizeProjectedOutletContracts(
    projectedMatch.layoutPlacements,
    projectedRouteCapability,
  );
  const projectedLayoutPlacements = Object.freeze(layoutPlacements.map(freezeLayoutPlacement));
  const finalOutletContract = outletContracts.at(-1);
  const verification = createProjectedCandidateVerification(
    canonicalUrlAuthority,
    projectedRouteCapability,
    projectedLayoutPlacements,
    outletContracts,
  );
  const matchedDeclarations = Object.freeze([
    ...projectedMatch.layoutDeclarations,
    projectedMatch.declaration,
  ]);
  return Object.freeze({
    [ROUTE_PROJECTED_CANDIDATE]: true,
    kind: "projectedCandidate",
    href: canonicalUrlAuthority.href,
    routeId: projectedRouteCapability.routeId,
    canonicalUrl() {
      return canonicalUrlAuthority;
    },
    route() {
      return projectedRouteCapability;
    },
    layouts() {
      return projectedLayoutPlacements;
    },
    outlet() {
      return finalOutletContract;
    },
    outlets() {
      return outletContracts;
    },
    warmup(trigger = "intent") {
      return createProjectedRoutePrefetchArtifact(this, trigger);
    },
    prefetch(trigger = "intent") {
      return this.warmup(trigger);
    },
    admission(facts = {}) {
      return createProjectedRouteAdmissionPlan(this, matchedDeclarations, projectRouteCandidate, facts);
    },
    verification() {
      return verification;
    },
  });
}

function selectProjectedMatch(
  nodes,
  routeAuthority,
  layoutPlacements,
  currentOutletId,
  layoutDeclarations,
) {
  let bestMatch = null;
  for (const node of nodes) {
    const candidate = projectNode(
      node,
      routeAuthority,
      layoutPlacements,
      currentOutletId,
      layoutDeclarations,
    );
    if (isBetterProjectedMatch(candidate, bestMatch)) {
      bestMatch = candidate;
    }
  }
  return bestMatch;
}

function projectNode(node, routeAuthority, layoutPlacements, currentOutletId, layoutDeclarations) {
  if (node.kind === "namespace") {
    return selectProjectedMatch(
      node.children,
      routeAuthority,
      layoutPlacements,
      currentOutletId,
      layoutDeclarations,
    );
  }
  if (node.kind === "route") {
    const leafLocation = node.reference.match(routeAuthority);
    if (leafLocation === null) {
      return null;
    }
    return {
      declaration: node.declaration,
      leafLocation,
      layoutPlacements,
      layoutDeclarations,
    };
  }
  const matchedLayoutParams = matchRoutePathPrefix(
    node.declaration.pattern,
    routeAuthority.pathname,
  );
  if (matchedLayoutParams === null) {
    return null;
  }
  const layoutLocation = node.reference.to({ params: matchedLayoutParams });
  const nextLayoutPlacements = Object.freeze([
    ...layoutPlacements,
    createProjectedLayoutPlacement(
      node.declaration,
      layoutLocation,
      node.outletId,
    ),
  ]);
  const nextLayoutDeclarations = Object.freeze([
    ...layoutDeclarations,
    node.declaration,
  ]);
  return selectProjectedMatch(
    node.children,
    routeAuthority,
    nextLayoutPlacements,
    node.outletId,
    nextLayoutDeclarations,
  );
}

function freezeLayoutPlacement(placement) {
  return placement;
}

function normalizeProjectionAuthority(routeAuthority) {
  if (isCanonicalUrlAuthority(routeAuthority)) {
    return routeAuthority;
  }
  if (isRawLocationAuthority(routeAuthority)) {
    return routeAuthority.canonical();
  }
  if (typeof routeAuthority === "string") {
    if (routeAuthority.length === 0) {
      throw new TypeError("routes.project(...) requires a non-empty href or path string");
    }
    try {
      return createCanonicalUrlAuthority(routeAuthority);
    } catch {
      return null;
    }
  }
  throw new TypeError(
    "routes.project(...) requires a local href string, raw location authority, or canonical url authority",
  );
}

function isBetterProjectedMatch(left, right) {
  if (left === null) {
    return false;
  }
  if (right === null) {
    return true;
  }
  return projectedMatchScore(left) > projectedMatchScore(right);
}

function projectedMatchScore(match) {
  return (
    match.layoutPlacements.length * 100 +
    match.leafLocation.descriptor().declarationPath.length
  );
}

export {
  attachProjectionRoot,
  createRouteLayoutReference,
};
