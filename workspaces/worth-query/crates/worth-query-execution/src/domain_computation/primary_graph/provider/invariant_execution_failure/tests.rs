use super::{map_custom_invariant_failure, map_invariant_failure, map_transaction_staging_failure};
use crate::domain_computation::WorthQueryCustomInvariantDenial;
use worth_relational::facade::transactions::{
    CustomInvariantFailureIdentity, CustomInvariantFailurePhase, CustomInvariantRuleId,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, InvariantViolationFields,
    ResultCustomInvariantFailureKind,
};

#[test]
fn custom_semantic_violation_preserves_owner_identity() {
    let identity = semantic_identity();
    let refusal = worth_relational::facade::runtime::CustomInvariantRefusal::DeclaredValue {
        value_identity: "tests.domain-refusal.v1".into(),
        bytes: b"17".to_vec(),
    };
    let failure = map_custom_invariant_failure(
        InvariantViolationFields::CustomInvariantViolation {
            identity: identity.clone(),
            refusal: refusal.clone(),
        },
        "custom semantic violation".to_owned(),
    )
    .unwrap();

    assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        failure.custom_invariant_denial(),
        Some(&WorthQueryCustomInvariantDenial::Violation { identity, refusal })
    );
    let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
            failure.clone(),
        );
    assert_eq!(
        denial.custom_invariant_denial(),
        failure.custom_invariant_denial()
    );
    assert_eq!(denial.invariant_execution_failure(), Some(&failure));
}

#[test]
fn custom_operational_failure_preserves_identity_phase_and_kind() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let identity = CustomInvariantFailureIdentity::new(semantic_identity());
        let failure = map_custom_invariant_failure(
            InvariantViolationFields::CustomInvariantFailure {
                identity: identity.clone(),
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
                detail: "hostile panic".to_owned(),
            },
            "custom invariant execution failed".to_owned(),
        )
        .unwrap();

        assert_eq!(
            failure.kind(),
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
        );
        assert_eq!(
            failure.custom_invariant_denial(),
            Some(&WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
            })
        );
    });
}

#[test]
fn built_in_invariant_preserves_relational_detail() {
    let failure = map_invariant_failure(
        InvariantViolationFields::None,
        "relation endpoint deletion leaves an incident edge".to_owned(),
    );

    assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected
    );
    assert_eq!(
        failure.detail(),
        "relation endpoint deletion leaves an incident edge"
    );
}

#[test]
fn invariant_materialization_refusal_is_not_exhaustion() {
    for (owner_denial, expected_detail) in [
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationAuthorityRequired,
                "Relational invariant transaction requires materialization authority",
            ),
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationModeMismatch,
                "Relational invariant transaction materialization mode does not match its intents",
            ),
        ] {
            let failure = map_transaction_staging_failure(owner_denial);
            let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
                failure,
            );
            let retained = denial.invariant_execution_failure().unwrap();
            assert_eq!(retained.kind(), crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected);
            assert_eq!(retained.posture(), crate::domain_computation::WorthQueryInvariantExecutionFailurePosture::Denied);
            assert_eq!(retained.detail(), expected_detail);
        }
}

fn semantic_identity() -> CustomInvariantSemanticIdentity {
    CustomInvariantSemanticIdentity {
        rule_id: CustomInvariantRuleId::new("query.custom-invariant"),
        semantic_version: CustomInvariantSemanticVersion::new(3, 7),
    }
}

#[test]
fn original_law_payload_survives_all_query_refusal_translations() {
    use crate::domain_computation::primary_graph::{
        application_attempt::provider_compare_denial::provider_session_denied,
        bootstrap_publication::map_bootstrap_commit_denial, WorthQueryApplicationCommitDenial,
        WorthQueryApplicationCommitDenialStage,
    };
    use crate::domain_computation::{
        WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
        WorthQueryProviderSessionProtocolStage, WorthQueryRelationLawDenial,
    };
    use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
    use worth_relational::facade::schema::ContractId;
    use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

    // Payload transport only: these coordinates make no claim of graph admission.
    // The installed public graph court proves actual law/identity derivation.
    let contract_id = ContractId::new("original-law-payload");
    let source = EntityReference::Existing(EntityId::new(PartitionId(2), 5, 3));
    let target = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId(2),
        kind_id: KindId(11),
        client_key: worth_relational::facade::symbols::ClientKey::raw("unpublished-target"),
    });
    let fields = InvariantViolationFields::Acyclicity {
        contract_id: contract_id.clone(),
        relation_kind_id: KindId(77),
        source: source.clone(),
        target: target.clone(),
    };
    let expected = Some(WorthQueryRelationLawDenial::Acyclicity {
        contract_id: &contract_id,
        relation_kind_id: KindId(77),
        source: &source,
        target: &target,
    });
    let original = law_transport_error(fields);
    let invariant = WorthQueryApplicationCommitDenial::invariant_execution_denied(
        WorthQueryApplicationCommitDenialStage::InvariantExecution,
        super::map_validation_failure(original.clone()),
    );
    assert_eq!(invariant.relation_law(), expected);
    let provider = provider_session_denied(
        WorthQueryProviderSessionFailure::new(
            WorthQueryProviderSessionDenialKind::ProviderRejected,
            WorthQueryProviderSessionProtocolStage::Commit,
            "opaque provider diagnostic",
            Default::default(),
        )
        .with_native_preparation_error(original.clone()),
    );
    assert_eq!(provider.relation_law(), expected);
    let bootstrap = map_bootstrap_commit_denial(original);
    assert_eq!(bootstrap.relation_law(), expected);
    assert!(invariant.custom_invariant_denial().is_none());
}

fn law_transport_error(
    fields: InvariantViolationFields,
) -> worth_relational::facade::transactions::TransactionCommitError {
    use worth_relational::facade::{
        diagnostics::DiagnosticCode,
        errors::{ErrorContext, ErrorOperation, RelationalSubsystem},
        transactions::{CommitConflict, ConflictClass, TransactionCommitError},
    };
    TransactionCommitError::conflict(CommitConflict {
        class: ConflictClass::InvariantViolation {
            code: DiagnosticCode::InvariantViolation,
            detail: "opaque native diagnostic".into(),
            fields,
        },
        code: DiagnosticCode::InvariantViolation,
        detail: "opaque native diagnostic".into(),
        context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate),
    })
}
