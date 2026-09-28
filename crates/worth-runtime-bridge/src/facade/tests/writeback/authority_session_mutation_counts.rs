use super::authority_session_mutation_evidence::execute_bridge_mutation_bundle;
use super::support::*;

#[test]
fn writeback_batch_mutation_authority_bundle_counts_naming_components() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:batch-mutation-authority-naming",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "batch-mutation-authority-naming",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");

    let component_a = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-naming:a",
        ),
        "batch-mutation-authority-naming:a",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-naming:a",
        ),
        "batch-mutation-authority-naming:a",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-naming:a",
        ),
    )
    .with_naming_mutation(
        crate::facade::BridgeNamingMutationBundle::attach_new_target(
            bridge_naming_attachment("persistent-name:first"),
            bridge_naming_target(
                crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts::entity(1, 2, 0),
            ),
            Some(bridge_naming_collection("Task")),
        ),
    );
    let component_b = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-naming:b",
        ),
        "batch-mutation-authority-naming:b",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-naming:b",
        ),
        "batch-mutation-authority-naming:b",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-naming:b",
        ),
    );

    let aggregate = crate::facade::BridgeBatchMutationAuthorityBundle::from_components(&[
        component_a,
        component_b,
    ])
    .expect("non-empty component set should aggregate");

    assert_eq!(aggregate.component_count(), 2);
    assert_eq!(aggregate.naming_mutation_count(), 1);
}

fn bridge_naming_attachment(value: &str) -> crate::facade::BridgeNamingAttachmentIdentity {
    crate::facade::BridgeNamingAttachmentIdentity::from_bridge_evidence(
        &crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(value),
    )
}

fn bridge_naming_target(
    parts: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
) -> crate::facade::BridgeNamingResolvedTargetIdentity {
    crate::facade::BridgeNamingResolvedTargetIdentity::from_relational_record(parts)
}

fn bridge_naming_collection(value: &str) -> crate::facade::BridgeNamingTargetCollection {
    crate::facade::BridgeNamingTargetCollection::new(value)
}

#[test]
fn writeback_batch_mutation_authority_bundle_counts_existing_and_symbolic_components() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:batch-mutation-authority-existing-symbolic",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "batch-mutation-authority-existing-symbolic",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");

    let component_a = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-existing-symbolic:a",
        ),
        "batch-mutation-authority-existing-symbolic:a",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-existing-symbolic:a",
        ),
        "batch-mutation-authority-existing-symbolic:a",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-existing-symbolic:a",
        ),
    )
    .with_existing_truth_binding(
        crate::facade::BridgeExistingTruthBindingBundle::direct_entity(
            crate::facade::BridgeExistingTruthBindingAuthoritativeIdentity::from_bridge_evidence(
                &crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "authority:task-existing",
                ),
            ),
            crate::facade::BridgeExistingTruthBindingResolvedTargetIdentity::from_relational_record(
                crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts::entity(1, 1, 0),
            ),
            Some(crate::facade::BridgeExistingTruthBindingTargetCollection::new("Task")),
        ),
    );
    let component_b = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-existing-symbolic:b",
        ),
        "batch-mutation-authority-existing-symbolic:b",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-existing-symbolic:b",
        ),
        "batch-mutation-authority-existing-symbolic:b",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-existing-symbolic:b",
        ),
    )
    .with_symbolic_target_reference(
        crate::facade::BridgeSymbolicTargetReferenceBundle::same_batch_target(
            crate::facade::BridgeSymbolicTargetSymbolIdentity::from_external_symbol_evidence(
                "draft-task",
            ),
            crate::facade::BridgeSymbolicTargetResolvedEntityIdentity::from_relational_record(
                crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts::entity(1, 4, 0),
            ),
            Some(crate::facade::BridgeSymbolicTargetCollection::new("Task")),
        ),
    );

    let aggregate = crate::facade::BridgeBatchMutationAuthorityBundle::from_components(&[
        component_a,
        component_b,
    ])
    .expect("non-empty component set should aggregate");

    assert_eq!(aggregate.component_count(), 2);
    assert_eq!(aggregate.existing_truth_binding_count(), 1);
    assert_eq!(aggregate.symbolic_target_reference_count(), 1);
    assert!(aggregate
        .aggregate_existing_truth_binding_digest()
        .is_some());
    assert!(aggregate
        .aggregate_symbolic_target_reference_digest()
        .is_some());
}
