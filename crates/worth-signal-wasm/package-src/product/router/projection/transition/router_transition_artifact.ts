import {
  ROUTE_TRANSITION_ARTIFACT,
} from "../../router_symbols.js";
import {
  createCanonicalDigest,
} from "../../url_authority/router_verification_packages.js";
import {
  isProjectedRoutePrefetchArtifact,
} from "./router_warmup_artifact.js";

async function createRouteTransitionArtifact(routes, currentOutcome, target, options = {}) {
  requireAdmittedCurrentOutcome(currentOutcome);
  const continuity = options.continuity ?? "refresh-immediately";
  const facts = options.facts ?? {};
  const targetResolution = await resolveTransitionTarget(routes, target, facts, options.source);
  const targetOutcome = targetResolution.outcome;
  const targetResources = selectTargetResources(targetResolution, targetOutcome);
  const visiblePolicy = classifyVisiblePolicy(currentOutcome, targetOutcome, continuity, targetResources);
  const visibleChangeSource = classifyVisibleChangeSource(targetResolution, targetOutcome, visiblePolicy);
  const pendingResourceNames = targetResources
    .filter((resource) => resource.current().status.kind === "pending")
    .map((resource) => resource.name);
  return Object.freeze({
    [ROUTE_TRANSITION_ARTIFACT]: true,
    kind: "routeTransition",
    currentRouteId: currentOutcome.routeId,
    currentHref: currentOutcome.href,
    targetRouteId: targetOutcome.routeId,
    targetHref: targetOutcome.href,
    target() {
      return targetOutcome;
    },
    diagnostics() {
      return Object.freeze({
        requestedSource: targetResolution.requestedSource,
        visibleChangeSource,
        visiblePolicy,
        continuity,
        pendingResourceNames: Object.freeze(pendingResourceNames),
      });
    },
    verification() {
      return Object.freeze({
        routeTransitionDigest: createCanonicalDigest("route-transition", {
          currentRouteId: currentOutcome.routeId,
          currentHref: currentOutcome.href,
          targetRouteId: targetOutcome.routeId,
          targetHref: targetOutcome.href,
          requestedSource: targetResolution.requestedSource,
          visibleChangeSource,
          visiblePolicy,
          continuity,
          pendingResourceNames,
          targetOutcomeDigest: targetOutcome.verification().routeOutcomeDigest,
        }),
      });
    },
  });
}

async function resolveTransitionTarget(routes, target, facts, requestedSource = "directNavigation") {
  if (isProjectedRoutePrefetchArtifact(target)) {
    return Object.freeze({
      requestedSource: "prefetchAdmission",
      prefetchedResources: target.resources(),
      outcome: await target.admit(facts),
    });
  }
  const firstOutcome = await routes.admit(target, facts);
  if (firstOutcome.kind === "redirect") {
    return Object.freeze({
      requestedSource: "redirect",
      outcome: await routes.admit(firstOutcome.artifact().href, facts),
    });
  }
  return Object.freeze({
    requestedSource,
    outcome: firstOutcome,
  });
}

function classifyVisiblePolicy(currentOutcome, targetOutcome, continuity, targetResources) {
  if (targetOutcome.kind !== "admitted") {
    return "preserve-current-route";
  }
  const hasPreservedTargetContinuity = targetResources.some((resource) => {
    const current = resource.current();
    return (
      current.status.kind === "pending"
      && "continuity" in current.status
      && current.status.continuity === "preservedVisibleValue"
      && current.diagnosticsSummary.current.hasVisibleValue === true
    );
  });
  if (hasPreservedTargetContinuity) {
    return "show-target-resource-continuity-while-pending";
  }
  if (continuity === "preserve-visible-while-pending") {
    return "preserve-current-route";
  }
  if (continuity === "preserve-visible-until-explicit-refresh") {
    return "preserve-current-route-until-explicit-refresh";
  }
  return "switch-to-target-route";
}

function selectTargetResources(targetResolution, targetOutcome) {
  if (targetOutcome.kind !== "admitted") {
    return [];
  }
  if (targetResolution.requestedSource === "prefetchAdmission") {
    return targetResolution.prefetchedResources;
  }
  return targetOutcome.route().resourceNames().map((name) => targetOutcome.route().resource(name));
}

function classifyVisibleChangeSource(targetResolution, targetOutcome, visiblePolicy) {
  if (visiblePolicy === "show-target-resource-continuity-while-pending") {
    return "resourceContinuityPreservation";
  }
  if (targetResolution.requestedSource === "prefetchAdmission") {
    return "prefetchAdmission";
  }
  if (targetResolution.requestedSource === "redirect") {
    return "redirect";
  }
  if (targetOutcome.kind === "redirect") {
    return "redirect";
  }
  return "directNavigation";
}

function requireAdmittedCurrentOutcome(value) {
  if (!value || value.kind !== "admitted") {
    throw new TypeError(
      "routes.transition(...) requires a current admitted route outcome as the visible route truth",
    );
  }
}

export {
  createRouteTransitionArtifact,
};
