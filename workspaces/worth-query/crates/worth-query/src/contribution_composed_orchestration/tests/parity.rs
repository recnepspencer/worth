use crate::contribution_composed_orchestration::{
    WorthQueryContributionComposedOrchestrationInput, WorthQueryContributionIntent,
};
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution, materialize_domain_capability_summary,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQuerySupportContributionAuthoring,
};

use super::support::{admitted_handle, standard_profile, target_for_envelope, ContributionInput};

#[test]
fn composed_orchestration_matches_explicit_declaration_bound_support_pipeline() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let progressed = match handle.declare_review_and_progress(ContributionInput::new("face-a")) {
        Ok(value) => value,
        Err(_) => panic!("expected progressed declaration"),
    };
    let target = handle.contribution_target(progressed.canonical_declaration());
    let envelope = match handle.orchestrate_envelope_from_progressed(progressed) {
        Ok(value) => value,
        Err(_) => panic!("expected envelope"),
    };
    let requested = WorthQuerySupportContributionAuthoring::declaration_traceability(
        "domain.traceability.face",
        "face selection remains traceable through declaration entry",
    )
    .bind_to_installed_target(target);
    let eligible = match evaluate_requested_domain_capability_contribution(requested) {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("expected eligible support contribution"),
    };
    let admitted = match admit_eligible_domain_capability_contribution(eligible) {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("expected admitted support contribution"),
    };
    let evidence_digest = admitted.admitted_for_reporting();
    let ready = match prepare_admitted_domain_capability_contribution_for_materialization(
        admitted,
        target_for_envelope(&handle, "face-a"),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("expected ready contribution"),
    };
    let expected_summary =
        materialize_domain_capability_summary(ready, standard_profile()).unwrap();

    let composed = match handle.orchestrate_declaration_with_contributions(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-a"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "face selection remains traceable through declaration entry",
                ),
            ))
            .materialize_summaries_with_profile(standard_profile()),
    ) {
        Ok(value) => value,
        Err(_) => panic!("expected composed contribution orchestration"),
    };

    assert_eq!(
        composed.envelope().envelope_digest(),
        envelope.envelope_digest()
    );
    assert_eq!(
        composed.classification(),
        crate::contribution_composed_orchestration::WorthQueryContributionComposedClassification::FullyAdmitted
    );
    assert_eq!(composed.contribution_composition().evidence().len(), 1);
    assert_eq!(
        composed.contribution_composition().evidence()[0].evidence_digest(),
        composed.contributions()[0].evidence().evidence_digest()
    );
    assert_eq!(
        composed.contributions()[0].evidence().evidence_digest(),
        evidence_digest
    );
    let summary = composed.contributions()[0]
        .summary()
        .expect("expected summary");
    assert_eq!(summary.outcome_kind(), expected_summary.outcome_kind());
    assert_eq!(
        summary.required_row_count(),
        expected_summary.required_row_count()
    );
    assert_eq!(
        summary.standard_row_count(),
        expected_summary.standard_row_count()
    );
    assert_eq!(
        summary.forensic_row_count(),
        expected_summary.forensic_row_count()
    );
    assert_eq!(composed.intent_results().len(), 1);
    assert_eq!(composed.rejected_intents().len(), 0);
    assert_eq!(
        composed.intent_results()[0]
            .aspect_record()
            .declaration_contract(),
        composed.envelope().aspect_contract()
    );
    assert_eq!(
        composed.intent_results()[0]
            .aspect_record()
            .declaration_coverage()
            .present(),
        composed.envelope().aspect_publication().present()
    );
}

#[test]
fn composed_request_digest_changes_with_declaration_and_contribution_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let first = handle.orchestrate_declaration_with_contributions_proof(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-d1"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "detail-one",
                ),
            )),
    );
    let second = handle.orchestrate_declaration_with_contributions_proof(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-d2"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "detail-two",
                ),
            )),
    );

    assert_ne!(first.request_digest(), second.request_digest());
}

#[test]
fn equivalent_intent_orderings_share_the_same_composition_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let left = handle
        .orchestrate_declaration_with_contributions(
            WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-e"))
                .with_contribution(WorthQueryContributionIntent::support(
                    WorthQuerySupportContributionAuthoring::declaration_traceability(
                        "domain.traceability.face",
                        "support detail",
                    ),
                ))
                .with_contribution(WorthQueryContributionIntent::support(
                    WorthQuerySupportContributionAuthoring::declaration_traceability(
                        "domain.traceability.edge",
                        "secondary support detail",
                    ),
                )),
        )
        .unwrap_or_else(|_| panic!("expected left composition"));
    let right = handle
        .orchestrate_declaration_with_contributions(
            WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-e"))
                .with_contribution(WorthQueryContributionIntent::support(
                    WorthQuerySupportContributionAuthoring::declaration_traceability(
                        "domain.traceability.edge",
                        "secondary support detail",
                    ),
                ))
                .with_contribution(WorthQueryContributionIntent::support(
                    WorthQuerySupportContributionAuthoring::declaration_traceability(
                        "domain.traceability.face",
                        "support detail",
                    ),
                )),
        )
        .unwrap_or_else(|_| panic!("expected right composition"));

    assert_eq!(
        left.composition_for_reporting(),
        right.composition_for_reporting()
    );
}
