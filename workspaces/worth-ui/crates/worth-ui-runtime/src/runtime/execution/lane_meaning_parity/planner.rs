use crate::runtime::execution::lane_meaning_parity::semantic_digest::{
    digest_identity_basis, digest_plan_family, digest_query_posture_entry,
    digest_query_rebind_entry, digest_references, WorthUiQueryReferenceSide,
};
use crate::runtime::planning::plan_equivalence::WorthUiExecutionPlanDigestor;
use crate::runtime::{
    WorthUiCrossLaneSemanticAuthority, WorthUiCrossLaneSemanticFamily,
    WorthUiCrossLaneSemanticReference, WorthUiExecutionPlan, WorthUiLaneMeaningParity,
    WorthUiLaneParityCertification, WorthUiLaneParityCounters, WorthUiLaneParityDenial,
    WorthUiLaneParityDenialReason, WorthUiLaneParityReport, WorthUiLaneTransitionParity,
    WorthUiNodeLifecycleTransition, WorthUiNodeReplacementPlan, WorthUiPlanNodeInputFamily,
    WorthUiQueryBindingComparison, WorthUiQueryBindingComparisonOutcome,
    WorthUiQueryLiveRebindOutcome, WorthUiQueryLiveRebindPlan, WorthUiRuntimeImpactNarrowing,
};

pub(crate) struct WorthUiLaneMeaningParityPlanner;

impl WorthUiLaneMeaningParityPlanner {
    pub(crate) fn certify(
        node_plan: &WorthUiNodeReplacementPlan,
        narrowing: &WorthUiRuntimeImpactNarrowing,
        active_plan: &WorthUiExecutionPlan,
        candidate_plan: &WorthUiExecutionPlan,
        query_comparison: &WorthUiQueryBindingComparison,
        query_rebind_plan: Option<&WorthUiQueryLiveRebindPlan>,
    ) -> Result<WorthUiLaneParityReport, WorthUiLaneParityDenial> {
        let mut counters = WorthUiLaneParityCounters::default();
        validate_inputs(
            node_plan,
            narrowing,
            query_comparison,
            query_rebind_plan,
            counters,
        )?;
        validate_lane_need(node_plan, narrowing, counters)?;
        validate_query_binding_meaning(query_comparison, query_rebind_plan, &mut counters)?;

        let references = shared_semantic_references(
            active_plan,
            candidate_plan,
            query_comparison,
            query_rebind_plan,
            &mut counters,
        );
        if references
            .iter()
            .any(|reference| !reference.preserves_meaning())
        {
            counters.record_semantic_mismatch();
            counters.record_visual_only_evidence_rejected();
            return Err(denial(
                WorthUiLaneParityDenialReason::VisualSimilarityWithoutSemanticParity,
                node_plan,
                counters,
            ));
        }

        let shared_meaning_parity = references
            .iter()
            .cloned()
            .map(WorthUiLaneMeaningParity::certified)
            .collect::<Vec<_>>();
        let transitions = node_plan
            .classifications()
            .iter()
            .filter(|classification| {
                classification.transition() == WorthUiNodeLifecycleTransition::LaneChange
            })
            .map(|classification| {
                counters.record_lane_transition();
                let mut transition_meaning_parity = shared_meaning_parity.clone();
                counters.record_semantic_reference();
                transition_meaning_parity.push(WorthUiLaneMeaningParity::certified(
                    identity_reference(classification.identity_basis()),
                ));
                WorthUiLaneTransitionParity::new(
                    classification.identity_basis(),
                    None,
                    None,
                    execution_mechanics_changed(active_plan, candidate_plan),
                    transition_meaning_parity,
                )
            })
            .collect::<Vec<_>>();
        let active_digest = WorthUiExecutionPlanDigestor::digest(active_plan).0.raw();
        let candidate_digest = WorthUiExecutionPlanDigestor::digest(candidate_plan).0.raw();
        let certification = WorthUiLaneParityCertification::new(
            node_plan.active_artifact_digest(),
            node_plan.candidate_artifact_digest(),
            active_digest,
            candidate_digest,
            digest_references(&references),
        );
        Ok(WorthUiLaneParityReport::new(
            certification,
            transitions,
            counters,
        ))
    }
}

fn identity_reference(identity_basis: &str) -> WorthUiCrossLaneSemanticReference {
    let digest = digest_identity_basis(identity_basis);
    WorthUiCrossLaneSemanticReference::new(
        WorthUiCrossLaneSemanticFamily::LaneChangeIdentity,
        identity_basis,
        digest,
        digest,
        WorthUiCrossLaneSemanticAuthority::DirectReferenceMatch,
    )
}

fn validate_inputs(
    node_plan: &WorthUiNodeReplacementPlan,
    narrowing: &WorthUiRuntimeImpactNarrowing,
    query_comparison: &WorthUiQueryBindingComparison,
    query_rebind_plan: Option<&WorthUiQueryLiveRebindPlan>,
    counters: WorthUiLaneParityCounters,
) -> Result<(), WorthUiLaneParityDenial> {
    if !node_plan.is_unambiguous() {
        return Err(denial(
            WorthUiLaneParityDenialReason::AmbiguousNodeReplacementPlan,
            node_plan,
            counters,
        ));
    }
    if !same_digest(
        node_plan,
        narrowing.active_artifact_digest(),
        narrowing.candidate_artifact_digest(),
    ) {
        return Err(denial(
            WorthUiLaneParityDenialReason::NarrowingDigestMismatch,
            node_plan,
            counters,
        ));
    }
    if !same_digest(
        node_plan,
        query_comparison.active_artifact_digest(),
        query_comparison.candidate_artifact_digest(),
    ) {
        return Err(denial(
            WorthUiLaneParityDenialReason::QueryComparisonDigestMismatch,
            node_plan,
            counters,
        ));
    }
    if let Some(rebind_plan) = query_rebind_plan {
        if !same_digest(
            node_plan,
            rebind_plan.active_artifact_digest(),
            rebind_plan.candidate_artifact_digest(),
        ) {
            return Err(denial(
                WorthUiLaneParityDenialReason::QueryRebindDigestMismatch,
                node_plan,
                counters,
            ));
        }
    }
    Ok(())
}

fn validate_lane_need(
    node_plan: &WorthUiNodeReplacementPlan,
    narrowing: &WorthUiRuntimeImpactNarrowing,
    counters: WorthUiLaneParityCounters,
) -> Result<(), WorthUiLaneParityDenial> {
    if narrowing
        .lane_impact()
        .is_some_and(|impact| impact.requires_lane_parity())
        && node_plan.counters().lane_changed_node_count() == 0
    {
        return Err(denial(
            WorthUiLaneParityDenialReason::MissingLaneChangeTransition,
            node_plan,
            counters,
        ));
    }
    Ok(())
}

fn validate_query_binding_meaning(
    comparison: &WorthUiQueryBindingComparison,
    rebind_plan: Option<&WorthUiQueryLiveRebindPlan>,
    counters: &mut WorthUiLaneParityCounters,
) -> Result<(), WorthUiLaneParityDenial> {
    for entry in comparison.entries() {
        counters.record_query_binding_checked();
        match entry.outcome() {
            WorthUiQueryBindingComparisonOutcome::PreserveMeaning => {}
            WorthUiQueryBindingComparisonOutcome::RebindRequired
            | WorthUiQueryBindingComparisonOutcome::MissingActiveBinding
            | WorthUiQueryBindingComparisonOutcome::MissingCandidateBinding => {
                let rebind = rebind_plan
                    .and_then(|plan| {
                        plan.binding_for_view_binding_id(entry.identity().view_binding_id())
                    })
                    .ok_or_else(|| {
                        counters.record_semantic_mismatch();
                        WorthUiLaneParityDenial::new(
                            WorthUiLaneParityDenialReason::QueryBindingChangedWithoutRebind,
                            comparison.active_artifact_digest(),
                            comparison.candidate_artifact_digest(),
                            *counters,
                        )
                    })?;
                counters.record_query_rebind_receipt();
                if matches!(rebind.outcome(), WorthUiQueryLiveRebindOutcome::Deny(_)) {
                    counters.record_semantic_mismatch();
                    return Err(WorthUiLaneParityDenial::new(
                        WorthUiLaneParityDenialReason::QueryRebindDenied,
                        comparison.active_artifact_digest(),
                        comparison.candidate_artifact_digest(),
                        *counters,
                    ));
                }
                if !query_rebind_outcome_matches_comparison(entry.outcome(), rebind.outcome()) {
                    counters.record_semantic_mismatch();
                    return Err(WorthUiLaneParityDenial::new(
                        WorthUiLaneParityDenialReason::QueryRebindOutcomeMismatch,
                        comparison.active_artifact_digest(),
                        comparison.candidate_artifact_digest(),
                        *counters,
                    ));
                }
            }
        }
    }
    Ok(())
}

fn query_rebind_outcome_matches_comparison(
    comparison: WorthUiQueryBindingComparisonOutcome,
    rebind: &WorthUiQueryLiveRebindOutcome,
) -> bool {
    matches!(
        (comparison, rebind),
        (
            WorthUiQueryBindingComparisonOutcome::RebindRequired,
            WorthUiQueryLiveRebindOutcome::Rebind(_)
        ) | (
            WorthUiQueryBindingComparisonOutcome::MissingActiveBinding,
            WorthUiQueryLiveRebindOutcome::Rebind(_)
        ) | (
            WorthUiQueryBindingComparisonOutcome::MissingCandidateBinding,
            WorthUiQueryLiveRebindOutcome::Retire(_)
        )
    )
}

fn shared_semantic_references(
    active_plan: &WorthUiExecutionPlan,
    candidate_plan: &WorthUiExecutionPlan,
    query_comparison: &WorthUiQueryBindingComparison,
    query_rebind_plan: Option<&WorthUiQueryLiveRebindPlan>,
    counters: &mut WorthUiLaneParityCounters,
) -> Vec<WorthUiCrossLaneSemanticReference> {
    let mut references = vec![
        plan_reference(
            WorthUiCrossLaneSemanticFamily::CommandMeaning,
            "command-plan-indexes",
            active_plan,
            candidate_plan,
            WorthUiPlanNodeInputFamily::Command,
            counters,
        ),
        plan_reference(
            WorthUiCrossLaneSemanticFamily::AccessibilityMeaning,
            "accessibility-plan-nodes",
            active_plan,
            candidate_plan,
            WorthUiPlanNodeInputFamily::Accessibility,
            counters,
        ),
        plan_reference(
            WorthUiCrossLaneSemanticFamily::DiagnosticsMeaning,
            "diagnostics-plan-nodes",
            active_plan,
            candidate_plan,
            WorthUiPlanNodeInputFamily::DiagnosticsRef,
            counters,
        ),
    ];
    for entry in query_comparison.entries() {
        let active = digest_query_posture_entry(entry, WorthUiQueryReferenceSide::Active);
        let (candidate, authority) = match entry.outcome() {
            WorthUiQueryBindingComparisonOutcome::PreserveMeaning => (
                digest_query_posture_entry(entry, WorthUiQueryReferenceSide::Candidate),
                WorthUiCrossLaneSemanticAuthority::DirectReferenceMatch,
            ),
            WorthUiQueryBindingComparisonOutcome::RebindRequired
            | WorthUiQueryBindingComparisonOutcome::MissingActiveBinding
            | WorthUiQueryBindingComparisonOutcome::MissingCandidateBinding => query_rebind_plan
                .and_then(|plan| {
                    plan.binding_for_view_binding_id(entry.identity().view_binding_id())
                })
                .map(|rebind| {
                    (
                        digest_query_rebind_entry(rebind),
                        WorthUiCrossLaneSemanticAuthority::QueryOwnedRebindReceipt,
                    )
                })
                .unwrap_or((0, WorthUiCrossLaneSemanticAuthority::DirectReferenceMatch)),
        };
        counters.record_semantic_reference();
        references.push(WorthUiCrossLaneSemanticReference::new(
            WorthUiCrossLaneSemanticFamily::QueryBindingMeaning,
            entry.identity().view_binding_id(),
            active,
            candidate,
            authority,
        ));
    }
    references.sort();
    references
}

fn plan_reference(
    family: WorthUiCrossLaneSemanticFamily,
    identity: &str,
    active_plan: &WorthUiExecutionPlan,
    candidate_plan: &WorthUiExecutionPlan,
    input_family: WorthUiPlanNodeInputFamily,
    counters: &mut WorthUiLaneParityCounters,
) -> WorthUiCrossLaneSemanticReference {
    counters.record_semantic_reference();
    WorthUiCrossLaneSemanticReference::new(
        family,
        identity,
        digest_plan_family(active_plan, input_family),
        digest_plan_family(candidate_plan, input_family),
        WorthUiCrossLaneSemanticAuthority::DirectReferenceMatch,
    )
}

fn execution_mechanics_changed(
    active_plan: &WorthUiExecutionPlan,
    candidate_plan: &WorthUiExecutionPlan,
) -> bool {
    WorthUiExecutionPlanDigestor::digest(active_plan).0
        != WorthUiExecutionPlanDigestor::digest(candidate_plan).0
}

fn same_digest(node_plan: &WorthUiNodeReplacementPlan, active: u64, candidate: u64) -> bool {
    node_plan.active_artifact_digest() == active
        && node_plan.candidate_artifact_digest() == candidate
}

fn denial(
    reason: WorthUiLaneParityDenialReason,
    node_plan: &WorthUiNodeReplacementPlan,
    counters: WorthUiLaneParityCounters,
) -> WorthUiLaneParityDenial {
    WorthUiLaneParityDenial::new(
        reason,
        node_plan.active_artifact_digest(),
        node_plan.candidate_artifact_digest(),
        counters,
    )
}
