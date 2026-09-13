use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableArtifactCrc32c, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalFreeSpaceMembershipBlock, PhysicalGeneration,
    PhysicalRecordFormatDeclaration, PhysicalTreeIdentity,
};
use worth_store_physical_integrity::{
    validate_free_space_header, validate_free_space_membership_block, PhysicalArtifactScope,
};

use super::membership_budget::{admit_membership_entries, MembershipProjectionFailure};
use super::{expected_range, source_input, source_range};
use crate::integrity_ingress::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

pub(crate) fn free_space_header(
    source: &ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    root: &DurablePhysicalRootManifest,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<DurableFreeSpaceManifestHeader, RecoveryIntegrityIngressRejection> {
    let generation = PhysicalGeneration::from_raw(root.generation())
        .map_err(|_| RecoveryIntegrityIngressRejection::ScopeMismatch)?;
    let tree = PhysicalTreeIdentity::new(root.tree_identity())
        .ok_or(RecoveryIntegrityIngressRejection::ScopeMismatch)?;
    let identity = FreeSpaceHeaderScopeIdentity::new(
        generation,
        tree,
        root.free_space_root(),
        DurableArtifactCrc32c::new(root.free_space_checksum()),
    );
    let expected_scope =
        PhysicalArtifactScope::free_space_header(store, format, identity, expected_range(format));
    let range =
        source_range(source).map_err(|rejection| trace.reject(expected_scope, rejection))?;
    let scope = PhysicalArtifactScope::free_space_header(store, format, identity, range);
    let validation = validate_free_space_header(source_input(source)?, scope).0;
    let attempt = IntegrityAdmittedRecoveryArtifact::bind_free_space_header(
        source,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    let admitted = attempt.into_outcome()?;
    let IntegrityAdmittedRecoveryArtifact::FreeSpaceHeader(admitted) = admitted else {
        unreachable!("free-space-header admission preserves its family")
    };
    let projection = admitted.project(trace.counters_mut());
    DurableFreeSpaceManifestHeader::new(
        projection.identity.generation().get(),
        projection.identity.tree().get(),
        projection.node_capacity,
        projection.segment_page_capacity,
        projection.entry_count,
        projection.next_segment,
        projection.next_page,
        projection.next_extent,
        projection.next_block,
        projection.root,
    )
    .ok_or(RecoveryIntegrityIngressRejection::NonCanonicalEncoding)
}

pub(crate) fn free_space_membership_block(
    source: &ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    tree: PhysicalTreeIdentity,
    reference: FreeSpaceBlockReference,
    capacity: u16,
    remaining_entries: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<PhysicalFreeSpaceMembershipBlock, MembershipProjectionFailure> {
    let identity = FreeSpaceMembershipBlockScopeIdentity::new(tree, reference);
    let expected_scope = PhysicalArtifactScope::free_space_membership_block(
        store,
        format,
        identity,
        expected_range(format),
    );
    let range =
        source_range(source).map_err(|rejection| trace.reject(expected_scope, rejection))?;
    let scope = PhysicalArtifactScope::free_space_membership_block(store, format, identity, range);
    let validation = validate_free_space_membership_block(source_input(source)?, scope).0;
    let attempt = IntegrityAdmittedRecoveryArtifact::bind_free_space_membership_block(
        source,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    let admitted = attempt.into_outcome()?;
    let IntegrityAdmittedRecoveryArtifact::FreeSpaceMembershipBlock(admitted) = admitted else {
        unreachable!("free-space-membership admission preserves its family")
    };
    let projection = admitted.project(trace.counters_mut());
    let entry_count = projection
        .entries
        .map_or_else(|| projection.children.map_or(0, <[_]>::len), <[_]>::len);
    admit_membership_entries(entry_count, capacity, remaining_entries)?;
    let block = if let Some(entries) = projection.entries {
        PhysicalFreeSpaceMembershipBlock::leaf(
            projection.identity.tree().get(),
            projection.generation,
            projection.block_identity,
            entries.to_vec(),
            capacity,
        )
    } else {
        PhysicalFreeSpaceMembershipBlock::branch(
            projection.identity.tree().get(),
            projection.generation,
            projection.block_identity,
            projection.level,
            projection.children.unwrap_or_default().to_vec(),
            capacity,
        )
    };
    block.ok_or_else(|| RecoveryIntegrityIngressRejection::NonCanonicalEncoding.into())
}
