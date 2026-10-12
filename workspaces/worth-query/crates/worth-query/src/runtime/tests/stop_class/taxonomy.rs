use super::super::support::*;
use super::completeness_support::{
    existing_binding, representative_runtime_stop_errors, status_value_touch,
};

#[test]
fn runtime_stop_class_taxonomy_covers_representative_runtime_error_variants() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for error in representative_runtime_stop_errors(request_execution) {
        match error.stop_class() {
            WorthQueryStopClass::Advancement { .. }
            | WorthQueryStopClass::ForeignPhase { .. }
            | WorthQueryStopClass::InstalledDomainAuthorityDenied { .. }
            | WorthQueryStopClass::MissingRuntimeComponent { .. }
            | WorthQueryStopClass::ExistingTruthAssertionDenied { .. }
            | WorthQueryStopClass::ExistingTruthProbeDenied { .. }
            | WorthQueryStopClass::MutationBindingDenied { .. }
            | WorthQueryStopClass::MutationContinuityDenied { .. }
            | WorthQueryStopClass::MutationContractDenied { .. }
            | WorthQueryStopClass::GraphCompositionDenied { .. }
            | WorthQueryStopClass::MutationNamingDenied { .. }
            | WorthQueryStopClass::MutationTargetReferenceDenied { .. }
            | WorthQueryStopClass::ReadCompositionDenied { .. }
            | WorthQueryStopClass::Workspace { .. }
            | WorthQueryStopClass::Program { .. }
            | WorthQueryStopClass::RuntimeLookupFailed { .. }
            | WorthQueryStopClass::MissingRuntimeArtifact { .. }
            | WorthQueryStopClass::SharedReadStaleBasis { .. }
            | WorthQueryStopClass::JournalReplayDenied { .. }
            | WorthQueryStopClass::RuntimeDeclarationFailed { .. }
            | WorthQueryStopClass::PreviewOperationEffectDenied { .. }
            | WorthQueryStopClass::SessionLabelCollision { .. }
            | WorthQueryStopClass::UnsupportedAuthorityRequirement { .. }
            | WorthQueryStopClass::ExistingTruthAssertionRequiresAuthorityLane { .. }
            | WorthQueryStopClass::IntentCommitDenied { .. }
            | WorthQueryStopClass::IntentExecutionRoutingFailed { .. }
            | WorthQueryStopClass::EffectPolicyDenied { .. }
            | WorthQueryStopClass::PreviewPromotionDenied { .. }
            | WorthQueryStopClass::FamilyAdmissionDenied { .. } => {}
        }
    }
}

#[test]
fn runtime_stop_class_ignores_runtime_declaration_message_wording() {
    let first = WorthQueryRuntimeError::ComputedDeclaration {
        view_name: "tasks.typed-stop-class".to_string(),
        stage: "shape-materialization",
        message: "first wording".to_string(),
    };
    let second = WorthQueryRuntimeError::ComputedDeclaration {
        view_name: "tasks.typed-stop-class".to_string(),
        stage: "shape-materialization",
        message: "second wording".to_string(),
    };

    for error in [first, second] {
        match error.stop_class() {
            WorthQueryStopClass::RuntimeDeclarationFailed {
                kind, name, stage, ..
            } => {
                assert_eq!(
                    kind,
                    WorthQueryRuntimeDeclarationFailureKind::ComputedDeclaration
                );
                assert_eq!(name, "tasks.typed-stop-class");
                assert_eq!(stage, "shape-materialization");
            }
            other => panic!("expected computed declaration stop class, got {other:?}"),
        }
    }
}

#[test]
fn runtime_stop_class_preserves_multiple_denial_kinds_within_the_same_family() {
    let binding = existing_binding();
    let status_touch = status_value_touch();

    let assertion_denials = [
        WorthQueryExistingTruthAssertionDenial::new(
            &binding,
            WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
            Some(status_touch.clone()),
            Some("\"open\"".to_string()),
            None,
            "missing asserted aspect",
        ),
        WorthQueryExistingTruthAssertionDenial::new(
            &binding,
            WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch,
            Some(status_touch),
            Some("\"open\"".to_string()),
            Some("\"closed\"".to_string()),
            "asserted value mismatch",
        ),
    ];

    for denial in assertion_denials {
        let expected_kind = denial.kind();
        match WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial).stop_class() {
            WorthQueryStopClass::ExistingTruthAssertionDenied { denial } => {
                assert_eq!(denial.kind(), expected_kind);
            }
            other => panic!("expected assertion-denial stop class, got {other:?}"),
        }
    }

    let read_denials = [
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ValidationDenied,
            "read validation failed",
        ),
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ExecutionDenied,
            "read execution failed",
        ),
    ];

    for denial in read_denials {
        let expected_kind = denial.kind().clone();
        match WorthQueryRuntimeError::ReadCompositionDenied(denial).stop_class() {
            WorthQueryStopClass::ReadCompositionDenied { denial } => {
                assert_eq!(denial.kind(), &expected_kind);
            }
            other => panic!("expected read-denial stop class, got {other:?}"),
        }
    }

    let naming_denials = [
        WorthQueryNamingMutationDenial::new(
            &WorthQueryNamingMutationIntent::attach_new_target(
                crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                    crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("attachment-1")
                        .expect("naming attachment authority label"),
                )
                .expect("naming attachment identity"),
            ),
            WorthQueryNamingMutationDenialKind::RequiresSameBatchTargetReference,
            "naming needs a same-batch target",
        ),
        WorthQueryNamingMutationDenial::new(
            &WorthQueryNamingMutationIntent::remove(
                crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                    crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("attachment-1")
                        .expect("naming attachment authority label"),
                )
                .expect("naming attachment identity"),
                crate::runtime::WorthQueryMutationAuthorityIdentity::naming_prior_authority(
                    crate::runtime::WorthQueryNamingPriorAuthorityLabel::new("attachment-1")
                        .expect("naming prior authority label"),
                )
                .expect("naming prior authority identity"),
            ),
            WorthQueryNamingMutationDenialKind::RequiresDeleteFamily,
            "naming requires delete family",
        ),
    ];

    for denial in naming_denials {
        let expected_kind = denial.kind();
        match WorthQueryRuntimeError::MutationNamingDenied(denial).stop_class() {
            WorthQueryStopClass::MutationNamingDenied { denial } => {
                assert_eq!(denial.kind(), expected_kind);
            }
            other => panic!("expected naming-denial stop class, got {other:?}"),
        }
    }
}
