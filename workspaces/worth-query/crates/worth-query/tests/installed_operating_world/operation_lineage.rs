use worth_foundational::{
    FoundationalBoundaryEvidenceContinuityAttachmentScope,
    FoundationalBoundaryEvidenceLineageOutcomeKind,
};
use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation, runtime};

use super::installed_operation_fixture::{
    lineage_workflow_workspace, workflow_workspace, GeometryDomain, LineageEvidenceScenario,
    ReadFamily, WorkflowRead,
};

#[test]
fn installed_lineage_is_executed_by_the_existing_identity_evolution_authority() {
    let cases = [
        (
            "preserved-lineage",
            LineageEvidenceScenario::PreservedIdentity,
            domain::WorthQueryOperationLineageContract::Preserve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity,
            1,
        ),
        (
            "singular-lineage",
            LineageEvidenceScenario::SingularSuccessor,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity,
            1,
        ),
        (
            "split-lineage",
            LineageEvidenceScenario::SplitSuccessors,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::PluralSuccessorPredecessor,
            2,
        ),
        (
            "merge-lineage",
            LineageEvidenceScenario::MergeSuccessor,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::MergeSuccessor,
            1,
        ),
        (
            "generated-lineage",
            LineageEvidenceScenario::GeneratedIdentity,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity,
            1,
        ),
        (
            "retired-lineage",
            LineageEvidenceScenario::RetiredIdentity,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::IdentityBreak,
            0,
        ),
        (
            "advisory-lineage",
            LineageEvidenceScenario::AdvisoryCorrespondence,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::AdvisoryCorrespondenceCandidate,
            0,
        ),
        (
            "ambiguous-lineage",
            LineageEvidenceScenario::AmbiguousCorrespondence,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::Ambiguity,
            0,
        ),
        (
            "broken-lineage",
            LineageEvidenceScenario::ContinuityBreak,
            domain::WorthQueryOperationLineageContract::Evolve,
            FoundationalBoundaryEvidenceLineageOutcomeKind::IdentityBreak,
            0,
        ),
    ];
    for (name, scenario, contract, expected_kind, expected_width) in cases {
        let mut workspace =
            lineage_workflow_workspace(name, contract, false, vec![scenario]).unwrap();
        let trace = execute(&mut workspace);
        let report = trace.lineage_report().unwrap();
        let evidence = &report.evidence()[0];

        assert_eq!(report.evidence().len(), 1);
        assert_eq!(report.counters().indexed_trace_stages, 4);
        assert_eq!(report.counters().indexed_effect_receipts, 1);
        assert_eq!(report.counters().stage_lookups, 1);
        assert_eq!(report.counters().outcome_contract_checks, 1);
        assert_eq!(report.counters().outcome_width, expected_width);
        assert_eq!(report.counters().unrelated_trace_scans, 0);
        assert_eq!(report.counters().unrelated_identity_scans, 0);
        assert_eq!(evidence.stage_identity(), "publish");
        assert!(!evidence.stage_receipt_identity().is_empty());
        assert_eq!(evidence.effect_receipt_identities().len(), 1);
        assert!(!evidence
            .outcome()
            .engine_artifact()
            .query_digest()
            .is_empty());
        assert!(!evidence
            .outcome()
            .engine_artifact()
            .basis_digest()
            .is_empty());
        if let Some(continuity) = evidence.outcome().continuity_evidence() {
            assert!(continuity.continuity_class().is_some());
        }
        assert_eq!(
            evidence.outcome().foundational_outcome_kind(),
            expected_kind
        );
        assert_eq!(
            evidence.foundational_lineage().outcome_kind(),
            expected_kind
        );
        assert_eq!(
            evidence
                .foundational_lineage()
                .materialized()
                .continuity_scope(),
            Some(FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel)
        );
        assert_eq!(
            evidence
                .foundational_lineage()
                .admit_current_basis()
                .payload()
                .target(),
            evidence.foundational_lineage().materialized().target()
        );
        assert!(!evidence
            .foundational_lineage()
            .subject_evidence_identity()
            .terminal_projection_for_reporting()
            .is_empty());
        assert!(!evidence
            .foundational_lineage()
            .source_basis_evidence_identity()
            .terminal_projection_for_reporting()
            .is_empty());
        assert!(!evidence
            .foundational_lineage()
            .receipt_evidence_identity()
            .terminal_projection_for_reporting()
            .is_empty());
        if let Some(naming) = evidence
            .outcome()
            .is_authoritative_continuity()
            .then(|| &trace.stage_receipts()[3].effect_evidence()[0])
            .and_then(domain::WorthQueryWorkflowEffectEvidence::mutation_receipt)
            .and_then(runtime::WorthQueryWriteReceipt::naming_mutation_evidence)
        {
            let intent = match (
                naming.target_authoritative_identity(),
                naming.resolved_target_entity_identity(),
            ) {
                (Some(target), _) => {
                    domain::WorthQueryPersistentNameIntent::from_executed_naming_attachment(
                        naming.attachment_identity().clone(),
                        target.clone(),
                    )
                }
                (None, Some(target)) => domain::WorthQueryPersistentNameIntent::
                    from_executed_generated_naming_attachment(
                        naming.attachment_identity().clone(),
                        target.clone(),
                    ),
                _ => continue,
            };
            let admission = trace.admit_persistent_name(0, intent).unwrap();
            assert_eq!(admission.lineage_report_identity(), report.identity());
        }
    }
}

#[test]
fn generated_identity_naming_requires_the_exact_generated_target() {
    let mut workspace = lineage_workflow_workspace(
        "generated-lineage-persistent-name",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::GeneratedIdentity],
    )
    .unwrap();
    let trace = execute(&mut workspace);
    let evidence = &trace.lineage_report().unwrap().evidence()[0];
    let naming = trace.stage_receipts()[3].effect_evidence()[0]
        .mutation_receipt()
        .unwrap()
        .naming_mutation_evidence()
        .expect("generated mutation must retain its naming receipt");
    let target = naming
        .resolved_target_entity_identity()
        .expect("generated naming must retain its exact entity target")
        .clone();

    assert_eq!(
        evidence.outcome().kind(),
        domain::InstalledIdentityEvolutionKind::GeneratedIdentity
    );
    let admitted = trace
        .admit_persistent_name(
            0,
            domain::WorthQueryPersistentNameIntent::from_executed_generated_naming_attachment(
                naming.attachment_identity().clone(),
                target,
            ),
        )
        .unwrap();
    assert_eq!(
        admitted.lineage_report_identity(),
        trace.lineage_report().unwrap().identity()
    );
}

#[test]
fn executor_lineage_must_exist_and_match_an_executable_installed_contract() {
    let mut missing = lineage_workflow_workspace(
        "lineage-missing-evidence",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::MutationWithoutLineage],
    )
    .unwrap();
    assert!(matches!(
        bind(&missing).admit_workflow_resources(crate::suite::installed_operation_fixture::execution_resource_request(), &missing).unwrap().reexecute(intent(), &mut missing),
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Completion(denial))
            if denial.kind() == domain::WorthQueryWorkflowCompletionDenialKind::LineageEvidence
                && denial.executed_effects().len() == 1
    ));

    let mut preserve = lineage_workflow_workspace(
        "lineage-preserve-needs-owner-evidence",
        domain::WorthQueryOperationLineageContract::Preserve,
        false,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    assert!(matches!(
        bind(&preserve).admit_workflow_resources(crate::suite::installed_operation_fixture::execution_resource_request(), &preserve).unwrap().reexecute(intent(), &mut preserve),
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Advance(denial))
            if denial.kind() == &domain::WorthQueryWorkflowAdvanceDenialKind::LineageEvidence
    ));

    let mut preserved = lineage_workflow_workspace(
        "lineage-preserve-exact-owner-evidence",
        domain::WorthQueryOperationLineageContract::Preserve,
        false,
        vec![LineageEvidenceScenario::PreservedIdentity],
    )
    .unwrap();
    let preserved_trace = execute(&mut preserved);
    assert_eq!(
        preserved_trace.lineage_report().unwrap().evidence()[0]
            .outcome()
            .kind(),
        domain::InstalledIdentityEvolutionKind::PreservedIdentity
    );

    let mut detached_workspace = workflow_workspace("lineage-detached-trace").unwrap();
    let detached = execute(&mut detached_workspace);
    assert!(matches!(
        detached.admit_persistent_name(
            0,
            domain::WorthQueryPersistentNameIntent::from_executed_naming_attachment(
                mutation_authority("detached-attachment"),
                mutation_authority("detached-target"),
            ),
        ),
        TransitionOutcome::Denied(domain::WorthQueryPersistentNameDenial::LineageMissing)
    ));
}

#[test]
fn persistent_naming_rejects_a_wrong_attachment_or_lineage_target() {
    let mut workspace = lineage_workflow_workspace(
        "lineage-hostile-persistent-name",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    let trace = execute(&mut workspace);
    let naming = trace
        .stage_receipts()
        .iter()
        .find(|stage| stage.stage_identity() == "publish")
        .unwrap()
        .effect_evidence()[0]
        .mutation_receipt()
        .unwrap()
        .naming_mutation_evidence()
        .unwrap();
    let exact_target = naming.target_authoritative_identity().unwrap().clone();

    assert!(matches!(
        trace.admit_persistent_name(
            0,
            domain::WorthQueryPersistentNameIntent::from_executed_naming_attachment(
                mutation_authority("wrong-attachment"),
                exact_target,
            ),
        ),
        TransitionOutcome::Denied(domain::WorthQueryPersistentNameDenial::NamingAttachmentMismatch)
    ));
    assert!(matches!(
        trace.admit_persistent_name(
            0,
            domain::WorthQueryPersistentNameIntent::from_executed_naming_attachment(
                naming.attachment_identity().clone(),
                mutation_authority("wrong-lineage-target"),
            ),
        ),
        TransitionOutcome::Denied(
            domain::WorthQueryPersistentNameDenial::TargetNotEstablishedByEvidence
        )
    ));
}

fn mutation_authority(label: &str) -> runtime::WorthQueryMutationAuthorityIdentity {
    runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
        runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(label).unwrap(),
    )
    .unwrap()
}

pub(super) fn execute(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQueryCompletedWorkflowTrace<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::MutationPreparationLaneWitness,
> {
    bind(workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .reexecute(intent(), workspace)
        .unwrap()
}

pub(super) fn bind(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::MutationPreparationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}

pub(super) fn intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("start", Value::NotRequired),
        Stage::new("right", Value::Text("start".into())),
        Stage::new("left", Value::Text("start".into())),
        Stage::new("publish", Value::Text("join".into())),
    ])
    .unwrap()
}
