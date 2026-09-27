use std::collections::BTreeSet;
use worth_ui_dsl::WorthUiSourceModuleId;

use crate::runtime::{
    WorthUiAccessibilityInvalidation, WorthUiAdmittedReplacementCandidate,
    WorthUiCommandBindingInvalidation, WorthUiCommandImpact, WorthUiImpactLookupCounters,
    WorthUiLaneImpactClassification, WorthUiQueryDependencyInvalidation,
    WorthUiRendererResourceInvalidation, WorthUiReplacementImpact,
    WorthUiReplacementImpactClassification, WorthUiRuntimeImpactNarrowing,
    WorthUiRuntimeImpactNarrowingDenial, WorthUiTokenInvalidation, WorthUiTokenThemeImpact,
};
use crate::source::{
    WorthUiArtifactHandle, WorthUiArtifactNodeKind, WorthUiArtifactSubtreeDigest,
    WorthUiIncrementalInvalidationBasis,
};

#[derive(Clone, Debug, Default)]
pub struct WorthUiRuntimeImpactNarrower;

impl WorthUiRuntimeImpactNarrower {
    pub fn narrow(
        classification: &WorthUiReplacementImpactClassification,
        admitted: &WorthUiAdmittedReplacementCandidate,
    ) -> Result<WorthUiRuntimeImpactNarrowing, WorthUiRuntimeImpactNarrowingDenial> {
        let mut counters = WorthUiImpactLookupCounters::default();
        counters.record_impact_classification_consumed();

        reject_mismatched_active_basis(classification, admitted, counters)?;
        reject_mismatched_candidate(classification, admitted, counters)?;

        counters.record_dependency_metadata_read();
        let basis = admitted
            .artifact_bundle()
            .dependency_metadata()
            .invalidation_basis();
        let affected_handles =
            affected_handles_from_impact(classification.impact(), basis, &mut counters);
        let affected_source_modules =
            affected_source_modules_from_handles(&affected_handles, basis, &mut counters);
        let affected_subtree_digests =
            affected_subtree_digests_from_handles(&affected_handles, basis, &mut counters);
        let query_dependency_invalidations =
            query_invalidations_from_handles(&affected_handles, basis, &mut counters);
        reject_affected_query_handles_without_query_invalidations(
            basis,
            &affected_handles,
            &query_dependency_invalidations,
            counters,
        )?;

        Ok(WorthUiRuntimeImpactNarrowing::new(
            super::WorthUiRuntimeImpactNarrowingInput {
                active_artifact_digest: classification.active_artifact_digest(),
                candidate_artifact_digest: classification.candidate_artifact_digest(),
                affected_source_modules,
                affected_handles: affected_handles.clone(),
                affected_subtree_digests,
                command_binding_invalidations: command_invalidations(
                    classification,
                    affected_handles.len(),
                ),
                token_invalidations: token_invalidations(classification, affected_handles.len()),
                accessibility_invalidation: WorthUiAccessibilityInvalidation::unchanged(),
                renderer_resource_invalidations: renderer_invalidations(&affected_handles),
                query_dependency_invalidations,
                lane_impact: lane_impact(classification),
                full_artifact_handle_count: basis.impact_metadata().full_artifact_handle_count(),
                counters,
            },
        ))
    }
}

fn reject_mismatched_active_basis(
    classification: &WorthUiReplacementImpactClassification,
    admitted: &WorthUiAdmittedReplacementCandidate,
    counters: WorthUiImpactLookupCounters,
) -> Result<(), WorthUiRuntimeImpactNarrowingDenial> {
    let admitted_active_artifact_digest = admitted.active_basis().artifact_digest();
    if classification.active_artifact_digest() == admitted_active_artifact_digest {
        Ok(())
    } else {
        Err(
            WorthUiRuntimeImpactNarrowingDenial::ClassificationActiveBasisMismatch {
                classification_active_artifact_digest: classification.active_artifact_digest(),
                admitted_active_artifact_digest,
                counters,
            },
        )
    }
}

fn reject_mismatched_candidate(
    classification: &WorthUiReplacementImpactClassification,
    admitted: &WorthUiAdmittedReplacementCandidate,
    counters: WorthUiImpactLookupCounters,
) -> Result<(), WorthUiRuntimeImpactNarrowingDenial> {
    let admitted_candidate_artifact_digest = admitted.artifact_bundle().artifact_digest().raw();
    if classification.candidate_artifact_digest() == admitted_candidate_artifact_digest {
        Ok(())
    } else {
        Err(
            WorthUiRuntimeImpactNarrowingDenial::ClassificationCandidateMismatch {
                classification_candidate_artifact_digest: classification
                    .candidate_artifact_digest(),
                admitted_candidate_artifact_digest,
                counters,
            },
        )
    }
}

fn affected_handles_from_impact(
    impact: &WorthUiReplacementImpact,
    basis: &WorthUiIncrementalInvalidationBasis,
    counters: &mut WorthUiImpactLookupCounters,
) -> Vec<WorthUiArtifactHandle> {
    match impact {
        WorthUiReplacementImpact::NoOp => Vec::new(),
        WorthUiReplacementImpact::LocalSubtree(scope)
        | WorthUiReplacementImpact::StructuralReplacement(scope)
        | WorthUiReplacementImpact::BroadReplacement(scope)
        | WorthUiReplacementImpact::LaneAffecting { scope, .. } => scope
            .impacted_handles()
            .iter()
            .flat_map(|handle| subtree_impact_handles(handle, basis, counters))
            .collect(),
    }
}

fn subtree_impact_handles(
    handle: &WorthUiArtifactHandle,
    basis: &WorthUiIncrementalInvalidationBasis,
    counters: &mut WorthUiImpactLookupCounters,
) -> Vec<WorthUiArtifactHandle> {
    counters.record_subtree_impact_lookup();
    let impact = basis.impact_metadata().impact_for_subtree(handle);
    if impact.impacted_handles().is_empty() {
        vec![handle.clone()]
    } else {
        impact.impacted_handles().to_vec()
    }
}

fn affected_source_modules_from_handles(
    handles: &[WorthUiArtifactHandle],
    basis: &WorthUiIncrementalInvalidationBasis,
    counters: &mut WorthUiImpactLookupCounters,
) -> Vec<WorthUiSourceModuleId> {
    let mut modules = BTreeSet::new();
    for handle in handles {
        modules.insert(handle.module_id().clone());
    }
    for module_id in &modules {
        counters.record_module_impact_lookup();
        let _ = basis.impact_metadata().impact_for_module(module_id);
    }
    modules.into_iter().collect()
}

fn affected_subtree_digests_from_handles(
    handles: &[WorthUiArtifactHandle],
    basis: &WorthUiIncrementalInvalidationBasis,
    counters: &mut WorthUiImpactLookupCounters,
) -> Vec<WorthUiArtifactSubtreeDigest> {
    handles
        .iter()
        .filter_map(|handle| {
            counters.record_subtree_digest_lookup();
            basis.dependency_graph().subtree_digest(handle)
        })
        .collect()
}

fn query_invalidations_from_handles(
    handles: &[WorthUiArtifactHandle],
    basis: &WorthUiIncrementalInvalidationBasis,
    counters: &mut WorthUiImpactLookupCounters,
) -> Vec<WorthUiQueryDependencyInvalidation> {
    handles
        .iter()
        .flat_map(|handle| {
            counters.record_runtime_hook_lookup();
            basis
                .dependency_graph()
                .runtime_hooks_for(handle)
                .iter()
                .map(WorthUiQueryDependencyInvalidation::from_runtime_hook)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn reject_affected_query_handles_without_query_invalidations(
    basis: &WorthUiIncrementalInvalidationBasis,
    affected_handles: &[WorthUiArtifactHandle],
    query_invalidations: &[WorthUiQueryDependencyInvalidation],
    counters: WorthUiImpactLookupCounters,
) -> Result<(), WorthUiRuntimeImpactNarrowingDenial> {
    let expected_runtime_hook_count = runtime_hook_count_from_dependency_metadata(basis);
    let affected_query_capable_node = affected_handles.iter().any(|handle| {
        matches!(
            handle.kind(),
            WorthUiArtifactNodeKind::Binding | WorthUiArtifactNodeKind::Surface
        )
    });
    if expected_runtime_hook_count == 0
        || !affected_query_capable_node
        || !query_invalidations.is_empty()
    {
        Ok(())
    } else {
        Err(
            WorthUiRuntimeImpactNarrowingDenial::QueryDependencyPostureMissing {
                expected_runtime_hook_count,
                observed_runtime_hook_count: query_invalidations.len(),
                counters,
            },
        )
    }
}

fn runtime_hook_count_from_dependency_metadata(
    basis: &WorthUiIncrementalInvalidationBasis,
) -> usize {
    basis
        .dependency_graph()
        .runtime_hooks()
        .values()
        .map(Vec::len)
        .sum()
}

fn command_invalidations(
    classification: &WorthUiReplacementImpactClassification,
    affected_handle_count: usize,
) -> Vec<WorthUiCommandBindingInvalidation> {
    match classification.command_impact() {
        WorthUiCommandImpact::Unchanged => Vec::new(),
        WorthUiCommandImpact::BindingOnly => {
            vec![WorthUiCommandBindingInvalidation::binding_only(
                affected_handle_count,
            )]
        }
    }
}

fn token_invalidations(
    classification: &WorthUiReplacementImpactClassification,
    affected_handle_count: usize,
) -> Vec<WorthUiTokenInvalidation> {
    match classification.token_theme_impact() {
        WorthUiTokenThemeImpact::Unchanged => Vec::new(),
        WorthUiTokenThemeImpact::ThemeOnly => {
            vec![WorthUiTokenInvalidation::theme_only(affected_handle_count)]
        }
    }
}

fn lane_impact(
    classification: &WorthUiReplacementImpactClassification,
) -> Option<WorthUiLaneImpactClassification> {
    match classification.impact() {
        WorthUiReplacementImpact::LaneAffecting { lane_impact, .. } => Some(lane_impact.clone()),
        WorthUiReplacementImpact::NoOp
        | WorthUiReplacementImpact::LocalSubtree(_)
        | WorthUiReplacementImpact::StructuralReplacement(_)
        | WorthUiReplacementImpact::BroadReplacement(_) => None,
    }
}

fn renderer_invalidations(
    affected_handles: &[WorthUiArtifactHandle],
) -> Vec<WorthUiRendererResourceInvalidation> {
    let resource_count = affected_handles
        .iter()
        .filter(|handle| matches!(handle.kind(), WorthUiArtifactNodeKind::Surface))
        .count();
    if resource_count == 0 {
        Vec::new()
    } else {
        vec![WorthUiRendererResourceInvalidation::narrowed_to_runtime_lane(resource_count)]
    }
}
