use crate::physical_runtime::layout::AdmittedDirectoryRetirement;
use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{DerivedFamilyRootDirectoryBinding, PersistedRecordIdentity};

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalReadProtectionDenial,
    ServingPhysicalRuntime,
};

pub enum PhysicalLayoutAppendFailure {
    RootProtection(PhysicalReadProtectionDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(crate::physical_runtime::ProvenNoEffectPhysicalMutation),
    Indeterminate(crate::physical_runtime::IndeterminatePhysicalMutation),
    MissingRecordIdentity,
    ExtraRecordIdentities,
}

impl std::fmt::Debug for PhysicalLayoutAppendFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootProtection(_) => f.write_str("RootProtection"),
            Self::Idempotency(_) => f.write_str("IdempotencyIssuance"),
            Self::Preparation(_) => f.write_str("Preparation"),
            Self::ProvenNoEffect(fate) => f
                .debug_tuple("ProvenNoEffect")
                .field(&fate.cause())
                .finish(),
            Self::Indeterminate(fate) => {
                f.debug_tuple("Indeterminate").field(&fate.stage()).finish()
            }
            Self::MissingRecordIdentity => f.write_str("MissingRecordIdentity"),
            Self::ExtraRecordIdentities => f.write_str("ExtraRecordIdentities"),
        }
    }
}

pub(super) enum LayoutAppendKind<'runtime> {
    BTreeNode(DurableArtifactFamilyId),
    DerivedDirectory {
        previous: Option<DerivedFamilyRootDirectoryBinding>,
        replaced_nodes: AdmittedDirectoryRetirement<'runtime>,
    },
}

pub(super) fn append_layout_record(
    runtime: &ServingPhysicalRuntime,
    kind: LayoutAppendKind<'_>,
    encoded: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<PersistedRecordIdentity, PhysicalLayoutAppendFailure> {
    // A byte-identical node or directory can be rebuilt after its earlier
    // record was retired. Its old completed idempotency key must not return
    // that now-unrouted RecordId. Each successful append advances this root
    // generation. Identical concurrent staging appends may share immutable
    // bytes, but only a directory publication can select a layout tree: its
    // full closure and exact predecessor are checked before that append.
    let selected_generation = runtime
        .records()
        .map_err(PhysicalLayoutAppendFailure::RootProtection)?
        .protected_root()
        .root()
        .generation()
        .get();
    let submission = runtime.record_submission();
    let material = layout_append_material(&kind, &encoded, selected_generation);
    let key = submission
        .issue_idempotency_key(material)
        .map_err(PhysicalLayoutAppendFailure::Idempotency)?;
    let request = PhysicalMutationRequest::platform_durable(key, deadline);
    let preparation = match kind {
        LayoutAppendKind::BTreeNode(family) => {
            submission.prepare_btree_node_append(encoded, family, placement, request)
        }
        LayoutAppendKind::DerivedDirectory {
            previous,
            replaced_nodes,
        } => submission.prepare_blob_derived_directory_append(
            encoded,
            previous,
            replaced_nodes,
            placement,
            request,
        ),
    };
    let completed = match preparation.into_raw() {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            match prepared.execute() {
                PhysicalMutationOutcome::Completed(completed) => completed,
                PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                    return Err(PhysicalLayoutAppendFailure::ProvenNoEffect(fate));
                }
                PhysicalMutationOutcome::Indeterminate(fate) => {
                    return Err(PhysicalLayoutAppendFailure::Indeterminate(fate));
                }
            }
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(completed)) => {
            completed
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(fate)) => {
            return Err(PhysicalLayoutAppendFailure::ProvenNoEffect(fate));
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
            return Err(PhysicalLayoutAppendFailure::Indeterminate(fate));
        }
        other => return Err(PhysicalLayoutAppendFailure::Preparation(other.into())),
    };
    match completed.persisted_records() {
        [record] => {
            runtime.release_blob_ingest_clean_frames(completed.completed_data_frame_coordinates());
            Ok(*record)
        }
        [] => Err(PhysicalLayoutAppendFailure::MissingRecordIdentity),
        _ => Err(PhysicalLayoutAppendFailure::ExtraRecordIdentities),
    }
}

fn layout_append_material(
    kind: &LayoutAppendKind<'_>,
    encoded: &[u8],
    selected_generation: u64,
) -> PhysicalMutationIdempotencyMaterial {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.layout.derived.append.v2");
    sha.update(selected_generation.to_le_bytes());
    sha.update([match kind {
        LayoutAppendKind::BTreeNode(_) => 1,
        LayoutAppendKind::DerivedDirectory { .. } => 2,
    }]);
    if let LayoutAppendKind::BTreeNode(family) = kind {
        let label = family.label().as_bytes();
        sha.update((label.len() as u64).to_le_bytes());
        sha.update(label);
    }
    if let LayoutAppendKind::DerivedDirectory {
        previous,
        replaced_nodes,
    } = &kind
    {
        sha.update([u8::from(previous.is_some())]);
        if let Some(binding) = previous {
            sha.update(binding.directory_record().allocation_epoch());
            sha.update(binding.directory_record().ordinal().to_le_bytes());
            if let Some(source) = binding.indexed_through_blob_publication() {
                sha.update([1]);
                sha.update(source.root_generation().to_le_bytes());
                sha.update(source.record().allocation_epoch());
                sha.update(source.record().ordinal().to_le_bytes());
                sha.update(source.encoded_digest());
            } else {
                sha.update([0]);
            }
        }
        for record in replaced_nodes.records() {
            sha.update(record.allocation_epoch());
            sha.update(record.ordinal().to_le_bytes());
        }
    }
    sha.update(encoded);
    PhysicalMutationIdempotencyMaterial::new(sha.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuilt_identical_layout_bytes_get_new_material_after_root_advances() {
        let catalog = LayoutAppendKind::BTreeNode(DurableArtifactFamilyId::BlobCatalog);
        let dedupe = LayoutAppendKind::BTreeNode(DurableArtifactFamilyId::DedupeIndex);
        let bytes = [5_u8; 64];
        let initial = layout_append_material(&catalog, &bytes, 7);
        assert_eq!(initial, layout_append_material(&catalog, &bytes, 7));
        assert_ne!(initial, layout_append_material(&catalog, &bytes, 8));
        assert_ne!(initial, layout_append_material(&dedupe, &bytes, 7));
    }
}
