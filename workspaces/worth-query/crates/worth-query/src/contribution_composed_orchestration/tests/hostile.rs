use crate::application::assert_declaration_aspect_projections;
use crate::contribution_composed_orchestration::{
    WorthQueryContributionComposedClassification, WorthQueryContributionComposedOrchestrationInput,
    WorthQueryContributionIntent,
};
use crate::domain_capabilities::{
    WorthQueryContinuityContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;

use super::support::{admitted_handle, ContributionInput, DeferredContributionInput};

#[test]
fn partial_composition_preserves_admitted_truth_when_later_intent_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let installed_authority = handle.installed_authority().witness_identity().clone();
    let checked = handle.orchestrate_declaration_with_contributions_checked(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-b"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "face selection remains traceable through declaration entry",
                ),
            ))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "",
                ),
            )),
    );

    match checked {
        crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationOutcome::Bound(
            composed,
        ) => {
            assert_eq!(
                composed.classification(),
                WorthQueryContributionComposedClassification::PartiallyAdmitted
            );
            assert_eq!(composed.contributions().len(), 1);
            assert_eq!(composed.rejected_intents().len(), 1);
            assert_eq!(
                composed
                    .composition()
                    .rejected_category_families()
                    .len(),
                1
            );
            assert_eq!(
                composed.contributions()[0].evidence().evidence_digest(),
                composed.contribution_composition().evidence()[0].evidence_digest()
            );
            assert_eq!(
                composed.installed_authority().witness_identity(),
                &installed_authority
            );
            assert!(composed.contributions().iter().all(|contribution| {
                contribution.installed_authority().witness_identity() == &installed_authority
            }));
        }
        _ => panic!("expected partial composition to remain inspectable as a bound artifact"),
    }
}

#[test]
fn package_contribution_policy_denies_before_admission_or_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use crate::contribution_composed_orchestration::{
        WorthQueryContributionComposedIntentClassification,
        WorthQueryContributionComposedIntentStageKind,
        WorthQueryContributionComposedOrchestrationOutcome,
    };

    let handle = super::support::admitted_handle_without_contributions(request_execution);
    let outcome = handle.orchestrate_declaration_with_contributions_checked(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new(
            "policy-denial",
        ))
        .with_contribution(WorthQueryContributionIntent::support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "domain.traceability.policy",
                "package policy remains authoritative",
            ),
        )),
    );

    let WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(_) = outcome else {
        panic!("an uninstalled contribution category must deny composition")
    };
    let proof = handle.orchestrate_declaration_with_contributions_proof(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new(
            "policy-denial",
        ))
        .with_contribution(WorthQueryContributionIntent::support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "domain.traceability.policy",
                "package policy remains authoritative",
            ),
        )),
    );
    let result = &proof.intent_results()[0];
    assert_eq!(
        result.classification(),
        WorthQueryContributionComposedIntentClassification::Denied
    );
    assert_eq!(
        result.evaluation().kind(),
        WorthQueryContributionComposedIntentStageKind::Denied
    );
    assert_eq!(
        result.admission().kind(),
        WorthQueryContributionComposedIntentStageKind::NotAttempted
    );
    assert_eq!(
        result.materialization().kind(),
        WorthQueryContributionComposedIntentStageKind::NotAttempted
    );
    assert!(result.contribution().is_none());
}

#[test]
fn no_admitted_contributions_stay_contribution_owned_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let checked = handle.orchestrate_declaration_with_contributions_checked(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-c"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "",
                ),
            )),
    );
    let ordinary = handle.orchestrate_declaration_with_contributions_outcome(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-c"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "",
                ),
            )),
    );

    assert!(matches!(
        checked,
        crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(_)
    ));
    match ordinary {
        WorthQueryOrdinaryOutcome::Denied(posture) => {
            assert_eq!(
                posture.checked_topology().contribution_composed_kind(),
                Some(
                    crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::ContributionDenied
                )
            );
        }
        _ => panic!("expected ordinary denial"),
    }
}

#[test]
fn declaration_deferred_stays_distinct_on_composed_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let checked = handle.orchestrate_declaration_with_contributions_checked(
        WorthQueryContributionComposedOrchestrationInput::new(DeferredContributionInput::new(
            "face-deferred",
        ))
        .with_contribution(WorthQueryContributionIntent::support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "domain.traceability.face",
                "deferred declaration still carries an attached contribution request",
            ),
        )),
    );
    let ordinary = handle.orchestrate_declaration_with_contributions_outcome(
        WorthQueryContributionComposedOrchestrationInput::new(DeferredContributionInput::new(
            "face-deferred",
        ))
        .with_contribution(WorthQueryContributionIntent::support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "domain.traceability.face",
                "deferred declaration still carries an attached contribution request",
            ),
        )),
    );

    assert!(matches!(
        checked,
        crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationOutcome::Deferred(_)
    ));
    match ordinary {
        WorthQueryOrdinaryOutcome::Deferred(posture) => {
            assert_eq!(
                posture.checked_topology().contribution_composed_kind(),
                Some(
                    crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Deferred
                )
            );
        }
        _ => panic!("expected ordinary deferred posture"),
    }
}

#[test]
fn proof_transcript_explains_partial_run() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let proof = handle.orchestrate_declaration_with_contributions_proof(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-proof"))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "proof-visible contribution",
                ),
            ))
            .with_contribution(WorthQueryContributionIntent::support(
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    "domain.traceability.face",
                    "",
                ),
            )),
    );

    assert_eq!(proof.intent_results().len(), 2);
    assert_eq!(
        proof.composition_classification(),
        Some(WorthQueryContributionComposedClassification::PartiallyAdmitted)
    );
    assert!(proof.declaration().envelope_digest().is_some());
    assert!(proof.declaration().aspect_contract().is_some());
    assert!(proof.declaration().aspect_publication().is_some());
    assert!(proof.contribution_digest().is_some());
    assert_eq!(
        proof.intent_results()[0].semantic_code(),
        "domain.traceability.face"
    );
    assert_eq!(
        proof.intent_results()[0].detail(),
        "proof-visible contribution"
    );
    assert_declaration_aspect_projections(
        proof.intent_results()[0]
            .aspect_record()
            .declaration_contract()
            .required(),
        &["selection.active_face", "selection.face"],
    );
    assert_eq!(
        proof.intent_results()[0]
            .aspect_record()
            .declaration_coverage()
            .present(),
        proof
            .declaration()
            .aspect_publication()
            .expect("declaration aspect publication should be retained")
            .present()
    );
}

#[test]
fn empty_contribution_request_is_rejected_as_unsupported() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let checked = handle.orchestrate_declaration_with_contributions_checked(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new("face-empty")),
    );

    assert!(matches!(
        checked,
        crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationOutcome::Unsupported(_)
    ));
}

#[test]
fn declaration_bound_continuity_is_admitted_on_contribution_composed_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution);
    let composed = match handle.orchestrate_declaration_with_contributions(
        WorthQueryContributionComposedOrchestrationInput::new(ContributionInput::new(
            "face-continuity",
        ))
        .with_contribution(WorthQueryContributionIntent::continuity(
            WorthQueryContinuityContributionAuthoring::preserved(
                "domain.continuity.face",
                "declaration retains canonical face continuity",
            ),
        )),
    ) {
        Ok(value) => value,
        Err(_) => panic!("expected declaration-bound continuity contribution to admit"),
    };

    assert_eq!(
        composed.classification(),
        WorthQueryContributionComposedClassification::FullyAdmitted
    );
    assert_eq!(composed.contributions().len(), 1);
    assert_eq!(
        composed.contribution_composition().composed_category_families(),
        &[crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage]
    );
    assert_eq!(
        composed.intent_results()[0].semantic_code(),
        "domain.continuity.face"
    );
}
