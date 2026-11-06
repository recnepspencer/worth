use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::super::test_support::{lower_runtime_target, ready, ready_payload, success};
use super::super::{
    materialize_graph_composition_capability_support_row,
    materialize_graph_composition_domain_invariant_denial,
    WorthQueryGraphCapabilityRuntimeSemantics, WorthQueryGraphInvariantDenialRuntimeSemantics,
    WorthQueryInvariantCapabilityContributionAuthoring,
    WorthQueryInvariantCapabilityContributionPayload,
    WorthQueryInvariantCapabilityContributionPosture,
};
use crate::runtime::WorthQueryGraphCompositionCapabilityClass;

#[test]
fn graph_capability_runtime_materializer_preserves_capability_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let row = success(materialize_graph_composition_capability_support_row(
        ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::graph_capability_gap(
                "graph.face_inner_loop_insertion",
                WorthQueryGraphCompositionCapabilityClass::TargetCombination,
                "graph.face_inner_loop_insertion",
                "topology substrate is unavailable",
            )?,
        )?,
    )?);

    assert_eq!(row.capability_family(), "graph.face_inner_loop_insertion");
    assert_eq!(
        row.capability_class(),
        WorthQueryGraphCompositionCapabilityClass::TargetCombination
    );

    Ok(())
}

#[test]
fn graph_capability_runtime_materializer_denies_missing_or_unsupported_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let missing =
        materialize_graph_composition_capability_support_row(ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::capability_gap(
                "graph.face_inner_loop_insertion",
                "topology substrate is unavailable",
            )?,
        )?)?;
    let unsupported =
        materialize_graph_composition_capability_support_row(ready_invariant_payload(
            WorthQueryInvariantCapabilityContributionPayload::with_graph_capability(
                WorthQueryInvariantCapabilityContributionPosture::InvariantDenial,
                "spatial.non_manifold_edge_split",
                "registered as a runtime invariant",
                Some(WorthQueryGraphCapabilityRuntimeSemantics::new(
                    "graph.face_inner_loop_insertion",
                    WorthQueryGraphCompositionCapabilityClass::TargetCombination,
                )),
            )?,
        )?)?;

    assert!(matches!(
        missing,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));
    assert!(matches!(
        unsupported,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn graph_invariant_denial_runtime_materializer_builds_query_denial(
) -> Result<(), IdentityDerivationDenial> {
    let denial = success(materialize_graph_composition_domain_invariant_denial(
        ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
                "spatial.non_manifold_edge_split",
                ["edges", "faces"],
                ["edge:12"],
                ["mixed_existing_and_symbolic_entity_identity_edges"],
                ["mixed_existing_target_followup_mutation"],
                "program-graph-1",
                "breadth-graph-1",
                "components=3;symbolic_entities=1;symbolic_relations=0;declared_collections=2;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                "spatial.non_manifold_edge_split",
                "result would introduce non-manifold topology",
            )?,
        )?,
    )?);

    assert_eq!(denial.invariant_family(), "spatial.non_manifold_edge_split");
    assert_eq!(
        denial.domain_invariant_summary().declared_collections(),
        &["edges".to_string(), "faces".to_string()]
    );
    assert_eq!(
        denial.failure_stage(),
        crate::runtime::WorthQueryGraphCompositionAdmissionTraceStage::DomainInvariantEvaluated
    );

    Ok(())
}

#[test]
fn graph_invariant_denial_runtime_materializer_denies_missing_or_wrong_posture(
) -> Result<(), IdentityDerivationDenial> {
    let missing =
        materialize_graph_composition_domain_invariant_denial(ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_denial(
                "spatial.non_manifold_edge_split",
                "result would introduce non-manifold topology",
            )?,
        )?)?;
    let wrong_posture = materialize_graph_composition_domain_invariant_denial(
        ready_invariant_payload(
            WorthQueryInvariantCapabilityContributionPayload::with_graph_invariant_denial(
                WorthQueryInvariantCapabilityContributionPosture::SupportSummary,
                "spatial.non_manifold_edge_split",
                "result would introduce non-manifold topology",
                Some(WorthQueryGraphInvariantDenialRuntimeSemantics::new(
                    "spatial.non_manifold_edge_split",
                    ["edges"],
                    ["edge:12"],
                    ["mixed_existing_and_symbolic_entity_identity_edges"],
                    ["mixed_existing_target_followup_mutation"],
                    "program-graph-1",
                    "breadth-graph-1",
                    "components=2;symbolic_entities=1;symbolic_relations=0;declared_collections=1;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                )?),
            )?,
        )?,
    )?;

    assert!(matches!(
        missing,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));
    assert!(matches!(
        wrong_posture,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn graph_invariant_denial_runtime_materializer_preserves_parity_and_difference(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_graph_composition_domain_invariant_denial(
        ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
                "spatial.non_manifold_edge_split",
                ["edges", "faces"],
                ["edge:12"],
                ["mixed_existing_and_symbolic_entity_identity_edges"],
                ["mixed_existing_target_followup_mutation"],
                "program-graph-1",
                "breadth-graph-1",
                "components=3;symbolic_entities=1;symbolic_relations=0;declared_collections=2;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                "spatial.non_manifold_edge_split",
                "result would introduce non-manifold topology",
            )?,
        )?,
    )?);
    let right = success(materialize_graph_composition_domain_invariant_denial(
        ready_invariant_payload(
            WorthQueryInvariantCapabilityContributionPayload::with_graph_invariant_denial(
                WorthQueryInvariantCapabilityContributionPosture::InvariantDenial,
                "spatial.non_manifold_edge_split",
                "result would introduce non-manifold topology",
                Some(WorthQueryGraphInvariantDenialRuntimeSemantics::new(
                    "spatial.non_manifold_edge_split",
                    ["edges", "faces"],
                    ["edge:12"],
                    ["mixed_existing_and_symbolic_entity_identity_edges"],
                    ["mixed_existing_target_followup_mutation"],
                    "program-graph-1",
                    "breadth-graph-1",
                    "components=3;symbolic_entities=1;symbolic_relations=0;declared_collections=2;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                )?),
            )?,
        )?,
    )?);
    let different = success(materialize_graph_composition_domain_invariant_denial(
        ready_invariant_capability(
            WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
                "spatial.closed_loop_self_intersection",
                ["edges", "faces"],
                ["edge:12"],
                ["mixed_existing_and_symbolic_entity_identity_edges"],
                ["mixed_existing_target_followup_mutation"],
                "program-graph-2",
                "breadth-graph-1",
                "components=3;symbolic_entities=1;symbolic_relations=0;declared_collections=2;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                "spatial.closed_loop_self_intersection",
                "result would introduce a self-intersecting loop",
            )?,
        )?,
    )?);

    assert_eq!(left.denial_digest(), right.denial_digest());
    assert_ne!(left.denial_digest(), different.denial_digest());

    Ok(())
}

fn ready_invariant_capability(
    authoring: WorthQueryInvariantCapabilityContributionAuthoring,
) -> Result<
    super::super::WorthQueryMaterializationReadyInvariantCapabilityContribution<
        super::super::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(authoring.bind_to_lower_runtime_boundary_target(
        lower_runtime_target("boundary-graph")?,
    ))?)
}

fn ready_invariant_payload(
    payload: WorthQueryInvariantCapabilityContributionPayload,
) -> Result<
    super::super::WorthQueryMaterializationReadyInvariantCapabilityContribution<
        super::super::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready_payload(
        lower_runtime_target("boundary-graph")?,
        payload,
    )?)
}
