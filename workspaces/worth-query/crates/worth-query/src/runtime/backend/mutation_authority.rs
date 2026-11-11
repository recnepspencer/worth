use super::writeback_effect_intent::WorthQueryBridgeWritebackEffectIntent;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryEntityIdentity, WorthQueryMutationKind, WorthQuerySnapshotIdentity,
    WorthQueryWorkspaceError,
};
use crate::runtime::{WorthQueryBackendAdmissibleMutation, WorthQueryExistingTruthBindingFamily};

use worth_runtime_bridge::facade::{
    AdmittedBridgeWritebackContract, BridgeDiagnosticsTier, BridgeExecutionPolicyClass,
    BridgeExistingTruthBindingAuthoritativeIdentity, BridgeExistingTruthBindingBundle,
    BridgeExistingTruthBindingResolvedTargetIdentity, BridgeExistingTruthBindingTargetCollection,
    BridgeIdentityEvidence, BridgeMutationAuthorityBundle, BridgeMutationSubject,
    BridgeMutationSubjectKind, BridgeMutationSubjectTarget, BridgeMutationSubjectTouch,
    BridgePolicyDeclaration, BridgePolicyDeclarationIdentity, BridgeRequestKind,
    BridgeRouteIdentity, BridgeWritebackAuthoritativeStateBasis, BridgeWritebackCausalityIdentity,
    BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass,
    BridgeWritebackEffectIdentity, BridgeWritebackEffectIntent, BridgeWritebackFamilyKind,
    BridgeWritebackIdempotenceClass, BridgeWritebackIdempotenceIdentity,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackStrategyClass,
    LoweredBridgeExecutionPolicy, RelationalBridgeSnapshotIdentityParts, RuntimeBridge,
    TruthCommitIdentity, TruthSnapshotIdentity,
};

pub(crate) struct WorthQueryBridgeMutationTarget<'a> {
    collection: &'a str,
    entity_identity: &'a WorthQueryEntityIdentity,
    mutation_kind: WorthQueryMutationKind,
}

impl<'a> WorthQueryBridgeMutationTarget<'a> {
    pub(crate) fn new(
        collection: &'a str,
        entity_identity: &'a WorthQueryEntityIdentity,
        mutation_kind: WorthQueryMutationKind,
    ) -> Self {
        Self {
            collection,
            entity_identity,
            mutation_kind,
        }
    }
}

pub(crate) fn build_bridge_authority_bundle(
    bridge: &RuntimeBridge,
    snapshot_identity: &WorthQuerySnapshotIdentity,
    mutation: &WorthQueryBackendAdmissibleMutation,
    target: WorthQueryBridgeMutationTarget<'_>,
) -> Result<BridgeMutationAuthorityBundle, WorthQueryWorkspaceError> {
    let writeback_identity = writeback_identity(mutation, &target, snapshot_identity)?;
    let writeback_bridge_identity = writeback_identity.bridge_evidence_identity();
    let (policy, contract) = admit_writeback_contract(bridge, &writeback_bridge_identity)?;
    let effect_intent = WorthQueryBridgeWritebackEffectIntent::from_admitted_mutation(
        BridgeWritebackEffectClass::AspectReconciliation,
        mutation,
    )?
    .into_bridge_intent();
    let mutation_subject = bridge_mutation_subject(mutation, &target, &effect_intent)?;
    let causality = bridge_writeback_causality(
        snapshot_identity,
        mutation_subject,
        &writeback_bridge_identity,
    )?;
    let effect = bridge.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::from_bridge_evidence(&writeback_bridge_identity),
        effect_intent,
    );
    let authoritative_state_basis = BridgeWritebackAuthoritativeStateBasis::from_effect(&effect);
    let idempotence = bridge.classify_writeback_idempotence(
        &effect,
        &policy,
        &authoritative_state_basis,
        BridgeWritebackIdempotenceIdentity::from_bridge_evidence(&writeback_bridge_identity),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let authority = bridge
        .execute_writeback_mutation_authority(&contract, &effect, &idempotence, &causality)
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    attach_existing_truth_binding(authority, mutation)
}

fn admit_writeback_contract(
    bridge: &RuntimeBridge,
    writeback_identity: &BridgeIdentityEvidence,
) -> Result<
    (
        LoweredBridgeExecutionPolicy,
        AdmittedBridgeWritebackContract,
    ),
    WorthQueryWorkspaceError,
> {
    let admitted_policy = bridge
        .admit_policy_declaration(BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::from_bridge_evidence(writeback_identity),
            BridgeRequestKind::Authoritative,
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeDiagnosticsTier::Standard,
            true,
            true,
        ))
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let policy = bridge.lower_admitted_policy(&admitted_policy);
    let contract = bridge
        .admit_writeback_declaration(
            BridgeWritebackDeclaration::writeback_capable(
                BridgeWritebackDeclarationIdentity::from_bridge_evidence(writeback_identity),
                BridgeRequestKind::Authoritative,
                BridgeWritebackFamilyKind::AspectReconciliation,
                BridgeWritebackEffectClass::AspectReconciliation,
                BridgeWritebackStrategyClass::AspectReconciliationCommit,
                BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &policy,
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    Ok((policy, contract))
}

fn bridge_writeback_causality(
    snapshot_identity: &WorthQuerySnapshotIdentity,
    mutation_subject: BridgeMutationSubject,
    writeback_identity: &BridgeIdentityEvidence,
) -> Result<BridgeWritebackNativeCausalityInputs, WorthQueryWorkspaceError> {
    let Some(snapshot_parts) = snapshot_identity.relational_parts() else {
        return Err(WorthQueryWorkspaceError::new(
            "bridge writeback authority requires a relational snapshot identity",
        ));
    };
    let evaluation_snapshot = RelationalBridgeSnapshotIdentityParts::new(
        snapshot_parts.snapshot_id(),
        snapshot_parts.version_id().saturating_sub(1),
    );
    Ok(BridgeWritebackNativeCausalityInputs::new(
        BridgeWritebackCausalityIdentity::from_bridge_evidence(writeback_identity),
        TruthCommitIdentity::from_relational_commit_id(snapshot_parts.version_id()),
        BridgeRouteIdentity::from_bridge_evidence(writeback_identity),
        TruthSnapshotIdentity::from_relational_snapshot(evaluation_snapshot),
        TruthSnapshotIdentity::from_relational_snapshot(snapshot_parts),
    )
    .bind_mutation_subject(mutation_subject))
}

fn bridge_mutation_subject(
    mutation: &WorthQueryBackendAdmissibleMutation,
    target: &WorthQueryBridgeMutationTarget<'_>,
    effect_intent: &BridgeWritebackEffectIntent,
) -> Result<BridgeMutationSubject, WorthQueryWorkspaceError> {
    let target_record = target
        .entity_identity
        .relational_record_parts()
        .ok_or_else(|| {
            WorthQueryWorkspaceError::new(
                "bridge mutation authority requires a Relational target identity projection",
            )
        })?;
    BridgeMutationSubject::from_effect_intent_and_touches(
        BridgeMutationSubjectTarget::new(
            target.collection,
            target_record,
            bridge_mutation_subject_kind(&target.mutation_kind),
        ),
        effect_intent,
        mutation
            .declared_aspect_touches()
            .into_iter()
            .map(bridge_mutation_subject_touch),
    )
    .map_err(|error| WorthQueryWorkspaceError::new(error.to_string()))
}

fn bridge_mutation_subject_touch(
    touch: crate::runtime::WorthQueryAspectTouch,
) -> BridgeMutationSubjectTouch {
    match touch.native_field_path() {
        Some(field_path) => BridgeMutationSubjectTouch::aspect_field_path(
            touch.native_aspect_key().clone(),
            field_path.clone(),
        ),
        None => BridgeMutationSubjectTouch::whole_aspect(touch.native_aspect_key().clone()),
    }
}

fn bridge_mutation_subject_kind(kind: &WorthQueryMutationKind) -> BridgeMutationSubjectKind {
    match kind {
        WorthQueryMutationKind::Created => BridgeMutationSubjectKind::Created,
        WorthQueryMutationKind::Updated => BridgeMutationSubjectKind::Updated,
        WorthQueryMutationKind::Deleted => BridgeMutationSubjectKind::Deleted,
    }
}

fn writeback_identity(
    mutation: &WorthQueryBackendAdmissibleMutation,
    target: &WorthQueryBridgeMutationTarget<'_>,
    snapshot_identity: &WorthQuerySnapshotIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let snapshot_evidence_identity = snapshot_identity.evidence_identity();
    let entity_evidence_identity = target.entity_identity.evidence_identity();
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority)
        .field_value(WorthQueryEvidenceTag::new("collection"), target.collection)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("entity"),
            &entity_evidence_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_identity"),
            &snapshot_evidence_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("mutation_family"),
            mutation.mutation_family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("mutation_kind"),
            mutation_kind_label(&target.mutation_kind),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("declared_aspect_operations"),
            mutation_operation_identity_parts(mutation),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("aspect_values"),
            mutation
                .admitted_aspect_values()
                .iter()
                .map(|aspect| aspect.terminal_digest_material()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("asserted_aspect_values"),
            mutation
                .asserted_admitted_aspect_values()
                .iter()
                .map(|aspect| aspect.terminal_digest_material()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("mutation_metadata"),
            mutation
                .mutation_metadata()
                .entries()
                .map(|(key, value)| format!("{}={}", key.as_str(), value.terminal_digest_text())),
        )
        .seal()
}

fn mutation_operation_identity_parts(
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> Vec<String> {
    mutation
        .declared_aspect_operations()
        .into_iter()
        .map(|operation| {
            format!(
                "{}:{}",
                operation.kind(),
                operation.aspect_touch().admitted_touch_digest_part()
            )
        })
        .collect()
}

fn mutation_kind_label(mutation_kind: &WorthQueryMutationKind) -> &'static str {
    match mutation_kind {
        WorthQueryMutationKind::Created => "created",
        WorthQueryMutationKind::Updated => "updated",
        WorthQueryMutationKind::Deleted => "deleted",
    }
}

fn attach_existing_truth_binding(
    bridge_authority: BridgeMutationAuthorityBundle,
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> Result<BridgeMutationAuthorityBundle, WorthQueryWorkspaceError> {
    let Some(binding) = mutation.existing_truth_binding() else {
        return Ok(bridge_authority);
    };
    let authoritative_identity =
        BridgeExistingTruthBindingAuthoritativeIdentity::from_bridge_evidence(
            &binding
                .authoritative_identity()
                .evidence_identity()
                .bridge_evidence_identity(),
        );
    let target_collection = binding
        .target_collection_identity()
        .map(|collection| BridgeExistingTruthBindingTargetCollection::new(collection.as_str()));
    let bundle = match binding.family() {
        WorthQueryExistingTruthBindingFamily::DirectEntityIdentity => {
            let resolved_entity_identity = bridge_existing_truth_resolved_target_identity(
                binding.resolved_entity_identity(),
                "bridge existing-truth entity binding requires a relational resolved entity identity",
            )?;
            BridgeExistingTruthBindingBundle::direct_entity(
                authoritative_identity,
                resolved_entity_identity,
                target_collection,
            )
        }
        WorthQueryExistingTruthBindingFamily::DirectRelationIdentity => {
            let resolved_relation_identity = bridge_existing_truth_resolved_target_identity(
                binding.resolved_relation_identity(),
                "bridge existing-truth relation binding requires a relational resolved relation identity",
            )?;
            BridgeExistingTruthBindingBundle::direct_relation(
                authoritative_identity,
                resolved_relation_identity,
                target_collection,
            )
        }
    };
    Ok(bridge_authority.with_existing_truth_binding(bundle))
}

fn bridge_existing_truth_resolved_target_identity(
    resolved_target_identity: &WorthQueryEntityIdentity,
    message: &'static str,
) -> Result<BridgeExistingTruthBindingResolvedTargetIdentity, WorthQueryWorkspaceError> {
    resolved_target_identity
        .relational_record_parts()
        .map(BridgeExistingTruthBindingResolvedTargetIdentity::from_relational_record)
        .ok_or_else(|| WorthQueryWorkspaceError::new(message))
}
