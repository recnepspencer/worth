use super::super::{
    BoundedMaterializationFrontierPreflight, FrontierBundleRoutePlanningError, FrontierPlanFamily,
    FrontierPlanningError, FrontierPlanningInput, FrontierPredictionDriftOutcome,
    FrontierRoutePlanningError, SerialFallbackBundleEvidence, SerialFallbackBundleRoutes,
    SerialFallbackEvidence, SerialFallbackRoute,
};
use super::{lower_frontier_bundle, lower_preflight_to_frontier_plan};

pub fn lower_preflight_to_serial_fallback_route(
    preflight: &BoundedMaterializationFrontierPreflight,
    evidence: &SerialFallbackEvidence,
) -> Result<SerialFallbackRoute, FrontierRoutePlanningError> {
    let preflight = preflight.as_preflight();
    let frontier_plan = lower_preflight_to_frontier_plan(preflight)?;
    if evidence.basis_digest() != preflight.basis().proof().digest().as_str() {
        return Err(FrontierRoutePlanningError::SerialFallbackUnavailable {
            posture_digest: evidence.route_posture_digest(&frontier_plan),
        });
    }
    if evidence.drift_outcome() == &FrontierPredictionDriftOutcome::DeniedByDrift {
        return Err(FrontierRoutePlanningError::PredictionDriftDenied {
            posture_digest: evidence.route_posture_digest(&frontier_plan),
        });
    }
    match frontier_plan.family() {
        FrontierPlanFamily::BoundedMaterialization => Ok(SerialFallbackRoute::new(
            preflight.clone(),
            frontier_plan,
            evidence,
        )),
        FrontierPlanFamily::OrderedCollection => {
            Err(FrontierRoutePlanningError::SerialFallbackUnavailable {
                posture_digest: evidence.route_posture_digest(&frontier_plan),
            })
        }
        _ => Err(FrontierRoutePlanningError::UnsupportedFrontierFamily),
    }
}

pub fn lower_preflight_bundle_to_serial_fallback_routes(
    preflights: &[BoundedMaterializationFrontierPreflight],
    evidence: &SerialFallbackBundleEvidence,
) -> Result<SerialFallbackBundleRoutes, FrontierBundleRoutePlanningError> {
    if preflights.is_empty() {
        return Err(FrontierBundleRoutePlanningError::UnsupportedBundleComposition);
    }
    if preflights.len() != evidence.route_evidences().len() {
        return Err(FrontierBundleRoutePlanningError::EvidenceCountMismatch {
            expected: preflights.len(),
            found: evidence.route_evidences().len(),
        });
    }

    let raw_preflights = preflights
        .iter()
        .map(|preflight| preflight.as_preflight().clone())
        .map(FrontierPlanningInput::from)
        .collect::<Vec<_>>();
    let bundle_plan = lower_frontier_bundle(&raw_preflights).map_err(|error| match error {
        FrontierPlanningError::UnsupportedFrontierFamily
        | FrontierPlanningError::UnsupportedBundleComposition => {
            FrontierBundleRoutePlanningError::UnsupportedBundleComposition
        }
        FrontierPlanningError::MixedBasisBundle {
            expected_basis_digest,
            found_basis_digest,
        } => FrontierBundleRoutePlanningError::MixedBasisBundle {
            expected_basis_digest: expected_basis_digest.as_str().to_string(),
            found_basis_digest: found_basis_digest.as_str().to_string(),
        },
    })?;
    if evidence.basis_digest() != bundle_plan.bundle_basis_digest().as_str() {
        return Err(FrontierBundleRoutePlanningError::MixedBasisBundle {
            expected_basis_digest: bundle_plan.bundle_basis_digest().as_str().to_string(),
            found_basis_digest: evidence.basis_digest().to_string(),
        });
    }

    let mut routes = Vec::with_capacity(preflights.len());
    for (index, (preflight, route_evidence)) in preflights
        .iter()
        .zip(evidence.route_evidences().iter())
        .enumerate()
    {
        let route = lower_preflight_to_serial_fallback_route(preflight, route_evidence).map_err(
            |error| FrontierBundleRoutePlanningError::RoutePlanningFailed {
                route_index: index,
                error,
            },
        )?;
        routes.push(route);
    }

    Ok(SerialFallbackBundleRoutes::new(
        bundle_plan.bundle_basis_digest().clone(),
        bundle_plan.counters().clone(),
        evidence,
        routes,
    ))
}
