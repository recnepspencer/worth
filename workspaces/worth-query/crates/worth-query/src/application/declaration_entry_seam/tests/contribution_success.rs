use crate::application::{
    WorthQueryDeclarationEntryContributionEvidence,
    WorthQueryDeclarationEntryContributionEvidenceSet, WorthQueryDeclarationEntryInspectionInput,
    WorthQueryDeclarationEntryReadinessRequest,
};

use super::support::{
    admitted_declaration_advisory, admitted_declaration_explanation, admitted_declaration_support,
    admitted_lower_runtime_aftermath, admitted_lower_runtime_explanation, admitted_plan,
    admitted_plan_aftermath, admitted_plan_continuity, admitted_plan_support,
    admitted_plan_workflow, bridge_signal_envelope, handle, lower_runtime_envelope,
    BridgeSignalFamily, Input,
};

#[test]
fn declaration_support_evidence_composes_into_readiness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let evidence = WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
        WorthQueryDeclarationEntryContributionEvidence::from(admitted_declaration_support(
            "family.readiness.digest",
            "support",
            "kept-by-domain",
        )),
    ]);

    let readiness = handle
        .try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
            WorthQueryDeclarationEntryReadinessRequest::base().with_contribution_evidence(evidence),
        )
        .unwrap_or_else(|_| panic!("readiness composition should succeed"));

    let composition = readiness
        .contribution_composition()
        .expect("composed readiness evidence should be present");
    assert_eq!(composition.evidence().len(), 1);
    assert_eq!(composition.composed_category_families().len(), 1);
}

#[test]
fn declaration_explanation_evidence_composes_into_unified_inspection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = bridge_signal_envelope(&handle, "edge:42");
    let canonical_declaration = envelope
        .foundational_evidence()
        .subject()
        .canonical_declaration()
        .clone();
    let row_digests = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                bridge_signal_envelope(&handle, "edge:42"),
            ),
        ))
        .unwrap_or_else(|_| panic!("baseline inspection should succeed"))
        .matching_row_digests()
        .to_vec();
    let evidence = WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
        WorthQueryDeclarationEntryContributionEvidence::from(admitted_declaration_explanation(
            &canonical_declaration,
            "explain",
            "needs context",
        )),
    ]);

    let inspection = handle
        .inspect_declaration_entry(
            WorthQueryDeclarationEntryInspectionInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
            )
            .with_contribution_evidence(evidence),
        )
        .unwrap_or_else(|_| panic!("inspection composition should succeed"));

    assert!(inspection.contribution_composition().is_some());
    assert_eq!(inspection.matching_row_digests(), row_digests.as_slice());
    assert!(inspection.relational_posture().is_none());
    assert!(inspection.bridge_posture().is_none());
    assert!(inspection.signal_posture().is_none());
}

#[test]
fn contribution_digest_changes_without_changing_matching_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = bridge_signal_envelope(&handle, "edge:42");
    let canonical_declaration = envelope
        .foundational_evidence()
        .subject()
        .canonical_declaration()
        .clone();
    let first = handle
        .inspect_declaration_entry(
            WorthQueryDeclarationEntryInspectionInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                    bridge_signal_envelope(&handle, "edge:42"),
                ),
            )
            .with_contribution_evidence(
                WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                    WorthQueryDeclarationEntryContributionEvidence::from(
                        admitted_declaration_advisory(&canonical_declaration, "advisory", "first"),
                    ),
                ]),
            ),
        )
        .unwrap_or_else(|_| panic!("first inspection should succeed"));
    let second = handle
        .inspect_declaration_entry(
            WorthQueryDeclarationEntryInspectionInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
            )
            .with_contribution_evidence(
                WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                    WorthQueryDeclarationEntryContributionEvidence::from(
                        admitted_declaration_advisory(&canonical_declaration, "advisory", "second"),
                    ),
                ]),
            ),
        )
        .unwrap_or_else(|_| panic!("second inspection should succeed"));

    assert_ne!(first.inspection_digest(), second.inspection_digest());
    assert_eq!(first.matching_row_digests(), second.matching_row_digests());
}

#[test]
fn readiness_digest_changes_while_baseline_rows_stay_intact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let baseline = handle.declaration_entry_readiness::<Input<BridgeSignalFamily>>();
    let composed = handle
        .try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
            WorthQueryDeclarationEntryReadinessRequest::base().with_contribution_evidence(
                WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                    WorthQueryDeclarationEntryContributionEvidence::from(
                        admitted_declaration_support(
                            "family.readiness.digest",
                            "support",
                            "composed",
                        ),
                    ),
                ]),
            ),
        )
        .unwrap_or_else(|_| panic!("composed readiness should succeed"));

    assert_ne!(baseline.readiness_digest(), composed.readiness_digest());
    assert_eq!(baseline.rows(), composed.rows());
}

#[test]
fn declaration_workflow_evidence_composes_when_admitted_plan_scope_is_present() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let plan = admitted_plan();
    let inspection = handle
        .inspect_declaration_entry(
            WorthQueryDeclarationEntryInspectionInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                    bridge_signal_envelope(&handle, "edge:42"),
                ),
            )
            .with_admitted_plan_scope(plan.clone())
            .with_contribution_evidence(
                WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                    WorthQueryDeclarationEntryContributionEvidence::from(admitted_plan_workflow(
                        &plan,
                        "workflow",
                        "preview-only",
                    )),
                ]),
            ),
        )
        .unwrap_or_else(|_| panic!("workflow evidence should compose with admitted-plan scope"));
    let composition = inspection
        .contribution_composition()
        .expect("workflow contribution composition should be present");
    assert!(composition.composed_category_families().contains(
        &crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview
    ));
}

#[test]
fn admitted_plan_bound_categories_compose_with_retained_plan_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let plan = admitted_plan();
    let readiness = handle
        .try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
            WorthQueryDeclarationEntryReadinessRequest::base()
                .for_retained_subject(
                    crate::application::WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(
                        crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                            bridge_signal_envelope(&handle, "edge:42"),
                        ),
                    ),
                )
                .with_admitted_plan_scope(plan.clone())
                .with_contribution_evidence(
                    WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                        WorthQueryDeclarationEntryContributionEvidence::from(
                            admitted_plan_support(&plan, "support", "plan-bound"),
                        ),
                        WorthQueryDeclarationEntryContributionEvidence::from(
                            admitted_plan_workflow(&plan, "workflow", "plan-preview"),
                        ),
                        WorthQueryDeclarationEntryContributionEvidence::from(
                            admitted_plan_continuity(&plan, "continuity", "lineage"),
                        ),
                        WorthQueryDeclarationEntryContributionEvidence::from(
                            admitted_plan_aftermath(&plan, "aftermath", "residue"),
                        ),
                    ]),
                ),
        )
        .unwrap_or_else(|_| panic!("admitted-plan-bound evidence should compose"));
    let composition = readiness
        .contribution_composition()
        .expect("plan-bound contribution composition should be present");
    assert_eq!(composition.evidence().len(), 4);
}

#[test]
fn lower_runtime_bound_categories_compose_with_retained_lower_runtime_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let lower_runtime = lower_runtime_envelope("lower-runtime:edge:42");
    let inspection = handle
        .inspect_declaration_entry(
            WorthQueryDeclarationEntryInspectionInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                    bridge_signal_envelope(&handle, "edge:42"),
                ),
            )
            .with_lower_runtime_boundary_scope(lower_runtime.clone())
            .with_contribution_evidence(
                WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                    WorthQueryDeclarationEntryContributionEvidence::from(
                        admitted_lower_runtime_explanation(
                            &lower_runtime,
                            "explain",
                            "cross-runtime context",
                        ),
                    ),
                    WorthQueryDeclarationEntryContributionEvidence::from(
                        admitted_lower_runtime_aftermath(
                            &lower_runtime,
                            "aftermath",
                            "runtime residue",
                        ),
                    ),
                ]),
            ),
        )
        .unwrap_or_else(|_| panic!("lower-runtime-bound evidence should compose"));
    let composition = inspection
        .contribution_composition()
        .expect("lower-runtime contribution composition should be present");
    assert_eq!(composition.evidence().len(), 2);
}
