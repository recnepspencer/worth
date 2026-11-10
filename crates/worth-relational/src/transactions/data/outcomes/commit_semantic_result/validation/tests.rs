mod proposed_families;
use super::*;
use crate::validation::engine::InvariantExecutionMetadata;

fn execution_metadata(ceiling: InvariantCostClass) -> InvariantExecutionMetadata {
    InvariantExecutionMetadata::new(
        InvariantExecutionPoint::SnapshotPublication,
        InvariantObservationKind::Committed,
        VersionId(1),
        VersionId(1),
        InvariantGroupSet::empty(),
        InvariantGroupSet::empty(),
        ceiling,
        InvariantExecutionDisposition::Executed,
        None,
        false,
        None,
        Vec::new(),
        None,
        None,
    )
}

#[test]
fn snapshot_pressure_ceiling_does_not_change_original_validation_metadata() {
    let global = execution_metadata(InvariantCostClass::Global);
    let partition = execution_metadata(InvariantCostClass::Partition);
    assert_ne!(global.max_cost(), partition.max_cost());
    assert_eq!(
        RelationalInvariantMetadataMeaning::from_original(&global),
        RelationalInvariantMetadataMeaning::from_original(&partition),
    );
}

#[test]
fn previous_snapshot_pressure_metadata_refuses_decode() {
    let current = RelationalInvariantMetadataMeaning::from_original(&execution_metadata(
        InvariantCostClass::Global,
    ));
    let mut previous = serde_json::to_value(&current).unwrap();
    previous.as_object_mut().unwrap().insert(
        "declared_cost".into(),
        serde_json::to_value(InvariantCostClass::Partition).unwrap(),
    );
    let bytes = rmp_serde::to_vec_named(&previous).unwrap();
    assert!(rmp_serde::from_slice::<RelationalInvariantMetadataMeaning>(&bytes).is_err());
    let bytes = rmp_serde::to_vec_named(&current).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<RelationalInvariantMetadataMeaning>(&bytes).unwrap(),
        current,
    );
}

fn composed_meaning() -> RelationalInvariantExecutionMeaning {
    use crate::validation::data::{
        InvariantCheckResult, InvariantRule, InvariantWitnessKey, TouchedStructuralSet,
    };
    use crate::validation::engine::InvariantExecutionResult;
    let mut metadata = RelationalInvariantMetadataMeaning::from_original(&execution_metadata(
        InvariantCostClass::Global,
    ));
    metadata.execution_point = InvariantExecutionPoint::ProposedState;
    let mut checks = Vec::new();
    let touched = TouchedStructuralSet::new(
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
    );
    for point in [
        InvariantExecutionPoint::CommitBoundary,
        InvariantExecutionPoint::MutationSensitive,
        InvariantExecutionPoint::SnapshotPublication,
    ] {
        let original = InvariantExecutionResult::executed(
            execution_metadata(InvariantCostClass::Global),
            vec![InvariantCheckResult {
                execution_point: point,
                failure_effect: InvariantFailureEffect::BlockCommit,
                rule: InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1)),
                witness: InvariantWitnessKey::pass(),
                groups: InvariantGroupSet::empty(),
                cost: InvariantCostClass::Touched,
                custom_provenance: None,
                verdict: InvariantVerdict::Pass,
            }],
        );
        let check = original.results().first().unwrap();
        checks.push(RelationalInvariantCheckMeaning {
            execution_point: check.execution_point,
            failure_effect: check.failure_effect,
            rule: check.rule.clone(),
            witness: check.witness.as_str().to_owned(),
            groups: check.groups,
            declared_cost: check.cost,
            custom_provenance: Some(RelationalInvariantCustomProvenanceMeaning {
                observation_kind: if point == InvariantExecutionPoint::CommitBoundary {
                    InvariantObservationKind::Committed
                } else {
                    InvariantObservationKind::Speculative
                },
                version: VersionId(2),
                current_version: metadata.current_version,
                touched: touched.provenance_summary(),
                counts: StructuralCountView::from_touched_scope(&touched),
                non_applicability: None,
            }),
            verdict: check.verdict.clone(),
        });
    }
    let decisions = checks
        .iter()
        .map(|check| RelationalInvariantDecisionMeaning {
            execution_point: check.execution_point,
            failure_effect: check.failure_effect,
            rule: check.rule.clone(),
            witness: check.witness.clone(),
            decision: InvariantDecisionKind::from_verdict(&check.verdict),
            groups: check.groups,
            declared_cost: check.declared_cost,
            custom_provenance_present: true,
        })
        .collect();
    RelationalInvariantExecutionMeaning {
        metadata,
        checks,
        decisions,
    }
}
#[test]
fn proposed_aggregate_preserves_each_declared_execution_point_and_read_view() {
    assert!(composed_meaning().is_consistent());
}
#[test]
fn proposed_aggregate_refuses_foreign_points_disagreeing_decisions_and_foreign_bases() {
    let original = composed_meaning();
    let mut foreign = original.clone();
    foreign.checks[0].execution_point = InvariantExecutionPoint::HarnessAudit;
    foreign.decisions[0].execution_point = InvariantExecutionPoint::HarnessAudit;
    assert!(!foreign.is_consistent());
    let mut decision = original.clone();
    decision.decisions[1].decision = InvariantDecisionKind::Violated;
    assert!(!decision.is_consistent());
    let mut basis = original;
    basis.checks[2]
        .custom_provenance
        .as_mut()
        .unwrap()
        .current_version = VersionId(99);
    assert!(!basis.is_consistent());
}
#[test]
fn proposed_aggregate_refuses_an_observation_from_another_stage() {
    let original = composed_meaning();
    for index in 0..original.checks.len() {
        let mut wrong = original.clone();
        let custom = wrong.checks[index].custom_provenance.as_mut().unwrap();
        custom.observation_kind = if custom.observation_kind == InvariantObservationKind::Committed
        {
            InvariantObservationKind::Speculative
        } else {
            InvariantObservationKind::Committed
        };
        assert!(!wrong.is_consistent(), "stage {index}");
    }
}
