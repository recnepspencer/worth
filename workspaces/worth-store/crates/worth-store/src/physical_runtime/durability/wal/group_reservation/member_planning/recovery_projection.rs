use sha2::Digest;
use worth_store_physical_format::{
    BlobRecordKind, PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryBlobSemantic,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
};

use crate::physical_runtime::PreparedPhysicalRootProjection;

use super::blob_semantic::blob_semantic;

pub(super) fn recovery_projection(
    data: &crate::physical_runtime::durability::WalBoundPhysicalDataPlan,
    root: &PreparedPhysicalRootProjection,
    blob_record_kind: Option<BlobRecordKind>,
    derived_directory_basis: Option<&crate::physical_runtime::PreparedDerivedDirectoryBasis>,
    prepared_bytes: &[Vec<u8>],
) -> PersistedPhysicalRecoveryProjection {
    let root_state = PersistedPhysicalRecoveryRootState::new(
        root.root_publication_allocation_bytes().get(),
        root.manifest_capacity_transition().identity_code(),
        root.recovery_manifest_capacity(),
        root.recovery_inline_allocations()
            .map(|allocation| {
                PersistedInlineSegmentAllocation::new(
                    allocation.segment(),
                    allocation.page_capacity(),
                    allocation.used_pages(),
                )
                .expect("ordinary planning retains a valid inline allocation")
            })
            .collect(),
        root.recovery_last_inline_record(),
        root.recovery_last_inline_segment(),
    )
    .expect("the prepared root retains an exact recovery root state");
    if let Some((copy, _)) = data.source_copy() {
        assert!(
            blob_record_kind.is_none(),
            "source-copy cannot carry blob semantic append"
        );
        let recipe = worth_store_physical_format::PersistedExtentCopyRecipe::new(
            copy.intent(),
            copy.durable_intent_lsn(),
            copy.intent_digest(),
        )
        .expect("sealed copy proof retains its canonical durable intent");
        return PersistedPhysicalRecoveryProjection::from_source_copy(
            root.source_root_generation(),
            root_state,
            recipe,
        )
        .expect("copy adoption names a current root no older than its protected source");
    }
    let frames = data
        .frames()
        .expect("non-copy plan carries admitted frames")
        .iter()
        .map(|frame| {
            let target = frame.basis().target();
            PersistedPhysicalRecoveryFrame::new(
                target.persisted_subject(),
                target.coordinate(),
                frame.bytes(),
            )
            .expect("the WAL-bound frame retains its exact admitted materialization")
        })
        .collect();
    let manifests = root
        .recovery_payload_manifests()
        .map(|(coordinate, bytes)| {
            PersistedPhysicalRecoveryManifest::new(*coordinate, bytes)
                .expect("the payload projection retains only governed recovery manifests")
        })
        .collect();
    let blob_semantic = if let Some(basis) = derived_directory_basis {
        let [bytes] = prepared_bytes else {
            unreachable!("classified directory append is singular")
        };
        let records = root.recovery_record_identities().collect::<Vec<_>>();
        let [record] = records.as_slice() else {
            unreachable!("classified directory append has one identity")
        };
        let binding = worth_store_physical_format::PersistedBlobSemanticRecordBinding::new(
            *record,
            sha2::Sha256::digest(bytes).into(),
            root.source_root_generation()
                .checked_add(1)
                .expect("successor generation"),
        )
        .expect("nonzero successor generation");
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(
            worth_store_physical_format::PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                binding,
                basis.indexed_through,
                basis.indexed_through_quarantine,
            ),
        )
    } else {
        blob_semantic(blob_record_kind, prepared_bytes, root)
    };
    let projection = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        root.source_root_generation(),
        root_state,
        root.recovery_record_identities().collect(),
        frames,
        root.recovery_placements().collect(),
        root.recovery_segment_updates().collect(),
        manifests,
        blob_semantic,
    )
    .expect("a WAL-bound physical mutation has one nonempty recovery projection");
    let projection = match derived_directory_basis {
        Some(basis) => projection
            .with_derived_retirement(
                basis.expected_previous,
                root.recovery_dropped_record_identities().collect(),
            )
            .expect("classified directory retains its exact retired record set"),
        None => projection,
    };
    match root.recovery_release_head_effect() {
        Some(effect) => projection
            .with_release_head_upsert(effect.clone())
            .expect("the fenced pre-WAL head effect binds this exact V3 descriptor"),
        None => projection,
    }
}
