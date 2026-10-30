use crate::application::{
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationFoundationalEvidenceClass, WorthQueryDeclarationFutureProjection,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};

use super::{
    aspects::{
        route_aspect_contract, route_aspect_fit, route_aspect_publication,
        route_aspect_publication_summary,
    },
    class::{
        WorthQueryDeclarationRouteIntentRequirement, WorthQueryDeclarationRouteMultiplicity,
        WorthQueryDeclarationRoutePlanClass, WorthQueryLowerAuthorityRouteFamily,
    },
    denial::{
        WorthQueryDeclarationRoutePlanDeferred, WorthQueryDeclarationRoutePlanDenialCause,
        WorthQueryDeclarationRoutePlanDenied, WorthQueryDeclarationRoutePlanFailed,
    },
    explain::WorthQueryDeclarationRoutePlanExplanation,
    input::WorthQueryDeclarationRoutePlanInput,
    intent::WorthQueryDeclarationRouteIntent,
    plan::WorthQueryDeclarationRoutePlan,
    route_set::{WorthQueryDeclarationRouteSegment, WorthQueryDeclarationRouteSet},
};

pub enum WorthQueryDeclarationRoutePlanChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Planned(WorthQueryDeclarationRoutePlan<D, I>),
    Deferred(WorthQueryDeclarationRoutePlanDeferred<D, I>),
    Denied(WorthQueryDeclarationRoutePlanDenied<D, I>),
    Failed(WorthQueryDeclarationRoutePlanFailed<D, I>),
}

pub(crate) fn worth_query_checked_declaration_route_plan<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    input: WorthQueryDeclarationRoutePlanInput<D, I>,
) -> WorthQueryDeclarationRoutePlanChecked<D, I> {
    let (progressed, evidence, route_intent) = input.into_parts();
    let route_contract = I::Family::route_contract();
    let future_projection =
        WorthQueryDeclarationFutureProjection::from_declaration(progressed.canonical_declaration());
    let route_aspect_contract = route_aspect_contract(progressed.aspect_contract());
    let route_aspect_fit = route_aspect_fit(evidence.aspect_coverage(), &route_aspect_contract);
    let route_aspect_publication =
        route_aspect_publication(&route_aspect_contract, evidence.aspect_coverage());

    if evidence.class() != WorthQueryDeclarationFoundationalEvidenceClass::ProgressionAdmitted {
        return WorthQueryDeclarationRoutePlanChecked::Denied(
            WorthQueryDeclarationRoutePlanDenied::new(
                progressed,
                evidence,
                route_intent,
                route_contract,
                WorthQueryDeclarationRoutePlanDenialCause::EvidenceMismatch,
            ),
        );
    }

    match route_aspect_fit {
        WorthQueryDeclarationAspectFit::Conflict => {
            return WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::AspectConflict,
                ),
            );
        }
        WorthQueryDeclarationAspectFit::MissingRequired => {
            return WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::MissingRequiredAspect,
                ),
            );
        }
        WorthQueryDeclarationAspectFit::Exact
        | WorthQueryDeclarationAspectFit::CompatibleSuperset
        | WorthQueryDeclarationAspectFit::Partial => {}
    }

    if progressed.canonical_declaration().handle_identity_digest()
        != evidence.handle_identity_digest()
    {
        return WorthQueryDeclarationRoutePlanChecked::Denied(
            WorthQueryDeclarationRoutePlanDenied::new(
                progressed,
                evidence,
                route_intent,
                route_contract,
                WorthQueryDeclarationRoutePlanDenialCause::WrongAdmittedWorld,
            ),
        );
    }

    if progressed.operating_context_identity_digest()
        != evidence.operating_context_identity_digest()
        || progressed.progression_digest() != evidence.progression_digest().unwrap_or_default()
        || format!(
            "{:?}",
            progressed.canonical_declaration().declaration_digest()
        ) != evidence.declaration_digest()
    {
        return WorthQueryDeclarationRoutePlanChecked::Denied(
            WorthQueryDeclarationRoutePlanDenied::new(
                progressed,
                evidence,
                route_intent,
                route_contract,
                WorthQueryDeclarationRoutePlanDenialCause::EvidenceMismatch,
            ),
        );
    }

    match route_contract.intent_requirement() {
        WorthQueryDeclarationRouteIntentRequirement::Required if route_intent.is_none() => {
            return WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::IntentRequired,
                ),
            );
        }
        WorthQueryDeclarationRouteIntentRequirement::Forbidden
            if route_intent
                .is_some_and(|intent| intent != WorthQueryDeclarationRouteIntent::Auto) =>
        {
            return WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::IntentForbidden,
                ),
            );
        }
        _ => {}
    }

    if route_intent == Some(WorthQueryDeclarationRouteIntent::DeferredRouting) {
        return if route_contract.can_defer() {
            WorthQueryDeclarationRoutePlanChecked::Deferred(
                WorthQueryDeclarationRoutePlanDeferred::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    "the declaration route remains explicitly deferred by caller intent",
                ),
            )
        } else {
            WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::IntentConflictsWithRouteContract,
                ),
            )
        };
    }

    let mut routes = Vec::new();
    for family in route_contract.allowed_route_families() {
        if !intent_allows_family(route_intent, *family) {
            continue;
        }
        if *family == WorthQueryLowerAuthorityRouteFamily::Mixed {
            return WorthQueryDeclarationRoutePlanChecked::Failed(
                WorthQueryDeclarationRoutePlanFailed::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    "mixed is a route-plan classification, not a concrete lower-authority route segment",
                ),
            );
        }
        routes.push(WorthQueryDeclarationRouteSegment::new(
            *family,
            format!(
                "{} admitted through {}",
                family.as_str(),
                route_contract.reason()
            ),
        ));
    }

    if routes.is_empty() {
        return if route_contract.can_defer() {
            WorthQueryDeclarationRoutePlanChecked::Deferred(
                WorthQueryDeclarationRoutePlanDeferred::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    "no concrete lower-authority route is active yet, so routing remains deferred",
                ),
            )
        } else {
            WorthQueryDeclarationRoutePlanChecked::Denied(
                WorthQueryDeclarationRoutePlanDenied::new(
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    WorthQueryDeclarationRoutePlanDenialCause::NoAllowedRoutes,
                ),
            )
        };
    }

    if route_contract.multiplicity() == WorthQueryDeclarationRouteMultiplicity::Singular
        && routes.len() > 1
    {
        return WorthQueryDeclarationRoutePlanChecked::Denied(
            WorthQueryDeclarationRoutePlanDenied::new(
                progressed,
                evidence,
                route_intent,
                route_contract,
                WorthQueryDeclarationRoutePlanDenialCause::ForbiddenRouteCombination,
            ),
        );
    }

    let class = classify_routes(&routes);
    let explanation = WorthQueryDeclarationRoutePlanExplanation::new(
        route_contract.reason(),
        vec![
            format!("family:{}", progressed.declaration_family_key()),
            format!(
                "operating_context:{}",
                progressed.operating_context_identity_digest()
            ),
            format!("progression:{}", progressed.progression_digest()),
            format!("route-aspect-fit:{route_aspect_fit:?}"),
            format!(
                "route-coverage-basis:{:?}",
                evidence.aspect_coverage_basis()
            ),
            format!(
                "route-publication:{}",
                route_aspect_publication_summary(&route_aspect_publication)
            ),
            format!(
                "future-projection-digest:{}",
                future_projection.projection_digest()
            ),
        ]
        .into_iter()
        .chain(future_projection.retained_facts())
        .collect(),
        routes
            .iter()
            .map(|route| route.reason().to_string())
            .collect(),
        route_intent.map(|intent| format!("intent:{} narrowed the route set", intent.as_str())),
    );
    WorthQueryDeclarationRoutePlanChecked::Planned(WorthQueryDeclarationRoutePlan::new(
        progressed,
        evidence,
        route_intent,
        WorthQueryDeclarationRouteSet::new(routes),
        class,
        route_contract.automation_requires_explicit_handoff(),
        route_aspect_contract,
        route_aspect_fit,
        route_aspect_publication,
        future_projection,
        explanation,
    ))
}

fn classify_routes(
    routes: &[WorthQueryDeclarationRouteSegment],
) -> WorthQueryDeclarationRoutePlanClass {
    if routes.len() > 1 {
        return WorthQueryDeclarationRoutePlanClass::Mixed;
    }
    match routes[0].family() {
        WorthQueryLowerAuthorityRouteFamily::Relational => {
            WorthQueryDeclarationRoutePlanClass::RelationalOnly
        }
        WorthQueryLowerAuthorityRouteFamily::Bridge => {
            WorthQueryDeclarationRoutePlanClass::BridgeOnly
        }
        WorthQueryLowerAuthorityRouteFamily::Signal => {
            WorthQueryDeclarationRoutePlanClass::SignalOnly
        }
        WorthQueryLowerAuthorityRouteFamily::Mixed
        | WorthQueryLowerAuthorityRouteFamily::Deferred
        | WorthQueryLowerAuthorityRouteFamily::Forbidden => {
            WorthQueryDeclarationRoutePlanClass::Mixed
        }
    }
}

fn intent_allows_family(
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
    family: WorthQueryLowerAuthorityRouteFamily,
) -> bool {
    match route_intent.unwrap_or(WorthQueryDeclarationRouteIntent::Auto) {
        WorthQueryDeclarationRouteIntent::Auto => true,
        WorthQueryDeclarationRouteIntent::RelationalOnly => {
            family == WorthQueryLowerAuthorityRouteFamily::Relational
        }
        WorthQueryDeclarationRouteIntent::BridgeOnly => {
            family == WorthQueryLowerAuthorityRouteFamily::Bridge
        }
        WorthQueryDeclarationRouteIntent::SignalOnly => {
            family == WorthQueryLowerAuthorityRouteFamily::Signal
        }
        WorthQueryDeclarationRouteIntent::RelationalAndBridge => {
            matches!(
                family,
                WorthQueryLowerAuthorityRouteFamily::Relational
                    | WorthQueryLowerAuthorityRouteFamily::Bridge
            )
        }
        WorthQueryDeclarationRouteIntent::DeferredRouting => false,
    }
}
