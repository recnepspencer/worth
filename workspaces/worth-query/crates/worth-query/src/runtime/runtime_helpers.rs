use super::*;
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQuerySnapshotIdentity,
};

mod subscription;

pub(super) use subscription::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorthQuerySameBatchSymbolicTarget {
    entity_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WorthQuerySameBatchSymbolicTargetKey {
    symbol: String,
}

impl WorthQuerySameBatchSymbolicTargetKey {
    pub(super) fn from_reference(reference: &WorthQuerySymbolicTargetReference) -> Self {
        Self::from_symbol(reference.symbol())
    }

    pub(super) fn from_symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
}

impl WorthQuerySameBatchSymbolicTarget {
    pub(super) fn new(
        entity_identity: WorthQueryEntityIdentity,
        target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    ) -> Self {
        Self {
            entity_identity,
            target_collection,
        }
    }

    pub(super) fn entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.entity_identity
    }

    pub(super) fn target_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub(super) fn terminal_target_collection_projection(&self) -> Option<&str> {
        self.target_collection_identity()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }
}

pub(super) fn admit_authority_requirements(
    requirements: &std::collections::BTreeSet<WorthQueryAuthorityRequirement>,
) -> Result<(), WorthQueryRuntimeError> {
    for requirement in requirements {
        match requirement {
            WorthQueryAuthorityRequirement::ReadOnly
            | WorthQueryAuthorityRequirement::Live
            | WorthQueryAuthorityRequirement::BranchLocal
            | WorthQueryAuthorityRequirement::Previewable
            | WorthQueryAuthorityRequirement::Writeback
            | WorthQueryAuthorityRequirement::ReplayRequired => {}
            WorthQueryAuthorityRequirement::Merge | WorthQueryAuthorityRequirement::Destructive => {
                return Err(WorthQueryRuntimeError::UnsupportedAuthorityRequirement(
                    requirement.clone(),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn live_subscription_error(
    view_name: &str,
    stage: &'static str,
    error: DeclarativeLiveQueryError,
) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage,
        message: format!("{error:?}"),
    }
}

pub(super) fn attach_symbolic_target_reference_to_receipt(
    mut receipt: WorthQueryMutationReceipt,
    reference: BridgeSymbolicTargetReferenceBundle,
) -> WorthQueryMutationReceipt {
    receipt.bridge_authority = receipt
        .bridge_authority
        .take()
        .map(|bundle| bundle.with_symbolic_target_reference(reference));
    receipt
}

pub(super) fn attach_naming_mutation_to_receipt(
    mut receipt: WorthQueryMutationReceipt,
    naming: BridgeNamingMutationBundle,
) -> WorthQueryMutationReceipt {
    receipt.bridge_authority = receipt
        .bridge_authority
        .take()
        .map(|bundle| bundle.with_naming_mutation(naming));
    receipt
}

pub(super) fn attach_continuity_mutation_to_receipt(
    mut receipt: WorthQueryMutationReceipt,
    continuity: BridgeContinuityMutationBundle,
) -> WorthQueryMutationReceipt {
    receipt.bridge_authority = receipt
        .bridge_authority
        .take()
        .map(|bundle| bundle.with_continuity_mutation(continuity));
    receipt
}

pub(super) fn synthetic_existing_assertion_receipt(
    binding: &WorthQueryExistingTruthTargetBinding,
    snapshot_identity: &WorthQuerySnapshotIdentity,
    declared_aspect_value_digest: Option<&crate::evidence_identity::WorthQueryEvidenceIdentity>,
) -> WorthQueryMutationReceipt {
    let assertion_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("binding"),
        binding.binding_digest(),
    )
    .field_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("snapshot_identity"),
        &snapshot_identity.evidence_identity(),
    )
    .optional_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("aspect_digest"),
        declared_aspect_value_digest,
    )
    .seal();
    WorthQueryMutationReceipt {
        commit_identity: WorthQueryCommitIdentity::preview(assertion_identity),
        snapshot_identity: snapshot_identity.clone(),
        deltas: Vec::new(),
        bridge_authority: None,
    }
}

pub(super) fn record_same_batch_symbolic_target(
    symbolic_targets: &mut BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    reference: Option<&WorthQuerySymbolicTargetReference>,
    declared_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
    receipt: &WorthQueryMutationReceipt,
) {
    if receipt
        .deltas
        .first()
        .is_none_or(|delta| delta.kind != WorthQueryMutationKind::Created)
    {
        return;
    }
    let Some(reference) = reference else {
        return;
    };
    let resolved_target = declared_collection
        .and_then(|collection| {
            let mut matches = receipt
                .deltas
                .iter()
                .filter(|delta| {
                    delta.kind == WorthQueryMutationKind::Created
                        && delta
                            .target_collection_identity()
                            .same_target_collection_as(collection)
                })
                .map(|delta| {
                    WorthQuerySameBatchSymbolicTarget::new(
                        delta.entity_identity.clone(),
                        Some(collection.clone()),
                    )
                })
                .collect::<Vec<_>>();
            (matches.len() == 1).then(|| matches.remove(0))
        })
        .or_else(|| {
            let (_, target_collection, target_entity_identity) =
                classify_receipt_mutation_summary(receipt);
            target_entity_identity
                .map(|identity| WorthQuerySameBatchSymbolicTarget::new(identity, target_collection))
        });
    let Some(resolved_target) = resolved_target else {
        return;
    };
    symbolic_targets.insert(
        WorthQuerySameBatchSymbolicTargetKey::from_reference(reference),
        resolved_target,
    );
}

pub(super) fn resolve_same_batch_symbolic_target(
    symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    reference: &WorthQuerySymbolicTargetReference,
) -> Result<WorthQuerySameBatchSymbolicTarget, WorthQueryRuntimeError> {
    let target_key = WorthQuerySameBatchSymbolicTargetKey::from_reference(reference);
    let Some(resolved_target) = symbolic_targets.get(&target_key) else {
        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
            WorthQuerySymbolicTargetReferenceDenial::new(
                reference,
                WorthQuerySymbolicTargetReferenceDenialKind::UnresolvedSameBatchTarget,
                format!(
                    "same-batch symbolic target `{}` was not declared earlier in this mutation batch",
                    reference.symbol()
                ),
            ),
        ));
    };
    if let Some(expected_collection) = reference.target_collection_identity()
        && resolved_target
            .target_collection_identity()
            .is_none_or(|resolved_collection| {
                !resolved_collection.same_target_collection_as(expected_collection)
            })
    {
        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                WorthQuerySymbolicTargetReferenceDenial::new(
                    reference,
                    WorthQuerySymbolicTargetReferenceDenialKind::CollectionMismatch,
                    format!(
                        "same-batch symbolic target `{}` resolved to collection `{}`, not `{expected_collection}`",
                        reference.symbol(),
                        resolved_target
                            .terminal_target_collection_projection()
                            .unwrap_or("unknown"),
                        expected_collection = expected_collection.as_str(),
                    ),
                ),
            ));
    }
    Ok(resolved_target.clone())
}

pub(super) fn resolve_symbolic_aspect_references(
    symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    mut aspects: Vec<WorthQueryAuthoredAspectMutation>,
    symbolic_aspect_references: &[WorthQuerySymbolicAspectReference],
) -> Result<Vec<WorthQueryAuthoredAspectMutation>, WorthQueryRuntimeError> {
    for reference in symbolic_aspect_references {
        let resolved_target =
            resolve_same_batch_symbolic_target(symbolic_targets, reference.reference())?;
        let Some(parts) = resolved_target
            .entity_identity()
            .relational_entity_record_parts()
        else {
            return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                WorthQuerySymbolicTargetReferenceDenial::new(
                    reference.reference(),
                    WorthQuerySymbolicTargetReferenceDenialKind::NonEntityReferenceTarget,
                    "symbolic aspect references require a concrete relational entity identity",
                ),
            ));
        };
        let native_entity = worth_foundational::facade::EntityId::new(
            worth_foundational::facade::PartitionId(parts.partition_id()),
            parts.local_slot(),
            parts.generation(),
        );
        aspects.push(WorthQueryAuthoredAspectMutation::new_set(
            reference.aspect_touch().clone(),
            worth_foundational::facade::AspectValue::EntityRef(native_entity),
        )?);
    }
    Ok(aspects)
}

pub(super) fn combined_batch_mutation_receipt(
    receipts: &[WorthQueryMutationReceipt],
) -> Result<WorthQueryMutationReceipt, WorthQueryRuntimeError> {
    let Some(last_receipt) = receipts.last() else {
        return Err(WorthQueryRuntimeError::Workspace(
            WorthQueryWorkspaceError::new("mutation batch must produce at least one write receipt"),
        ));
    };
    let commit_identity = WorthQueryCommitIdentity::preview(
        crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::BatchWriteReceipt,
        )
        .field_value_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("component_commit_identity"),
            receipts.iter().map(|receipt| {
                receipt
                    .commit_identity
                    .evidence_identity()
                    .reporting_projection()
                    .to_string()
            }),
        )
        .seal(),
    );
    let deltas = receipts
        .iter()
        .flat_map(|receipt| receipt.deltas.iter().cloned())
        .collect::<Vec<_>>();
    Ok(WorthQueryMutationReceipt {
        commit_identity,
        snapshot_identity: last_receipt.snapshot_identity.clone(),
        deltas,
        bridge_authority: None,
    })
}

pub(super) fn classify_receipt_mutation_summary(
    receipt: &WorthQueryMutationReceipt,
) -> (
    WorthQueryMutationFamily,
    Option<WorthQueryMutationTargetCollectionIdentity>,
    Option<WorthQueryEntityIdentity>,
) {
    let mutation_family = receipt
        .deltas
        .first()
        .map(|delta| match delta.kind {
            WorthQueryMutationKind::Created => WorthQueryMutationFamily::Insert,
            WorthQueryMutationKind::Updated => WorthQueryMutationFamily::Update,
            WorthQueryMutationKind::Deleted => WorthQueryMutationFamily::Delete,
        })
        .unwrap_or(WorthQueryMutationFamily::Update);
    let mut collections = receipt
        .deltas
        .iter()
        .map(|delta| delta.target_collection_identity().clone())
        .collect::<Vec<_>>();
    collections.sort();
    collections.dedup_by(|left, right| left.same_target_collection_as(right));
    let mut entity_identities = receipt
        .deltas
        .iter()
        .map(|delta| delta.entity_identity.clone())
        .collect::<Vec<_>>();
    entity_identities.sort();
    entity_identities.dedup();
    (
        mutation_family,
        (collections.len() == 1).then(|| collections[0].clone()),
        (entity_identities.len() == 1).then(|| entity_identities[0].clone()),
    )
}
