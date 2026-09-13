use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalRecordFormatDeclaration,
    PhysicalSegmentMembershipBlock, PhysicalTreeIdentity, SegmentManifestBlockReference,
    SegmentMembershipBlockScopeIdentity,
};
use worth_store_physical_integrity::{validate_segment_membership_block, PhysicalArtifactScope};

use super::membership_budget::{admit_membership_entries, MembershipProjectionFailure};
use super::{expected_range, source_input, source_range};
use crate::integrity_ingress::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

pub(crate) fn segment_membership_block(
    source: &ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    tree: PhysicalTreeIdentity,
    reference: SegmentManifestBlockReference,
    capacity: u16,
    remaining_entries: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<PhysicalSegmentMembershipBlock, MembershipProjectionFailure> {
    let identity = SegmentMembershipBlockScopeIdentity::new(tree, reference);
    let expected_scope = PhysicalArtifactScope::segment_membership_block(
        store,
        format,
        identity,
        expected_range(format),
    );
    let range =
        source_range(source).map_err(|rejection| trace.reject(expected_scope, rejection))?;
    let scope = PhysicalArtifactScope::segment_membership_block(store, format, identity, range);
    let validation = validate_segment_membership_block(source_input(source)?, scope).0;
    let attempt = IntegrityAdmittedRecoveryArtifact::bind_segment_membership_block(
        source,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    let admitted = attempt.into_outcome()?;
    let IntegrityAdmittedRecoveryArtifact::SegmentMembershipBlock(admitted) = admitted else {
        unreachable!("segment-membership admission preserves its family")
    };
    let projection = admitted.project(trace.counters_mut());
    let entry_count = projection
        .entries
        .map_or_else(|| projection.children.map_or(0, <[_]>::len), <[_]>::len);
    admit_membership_entries(entry_count, capacity, remaining_entries)?;
    let block = if let Some(entries) = projection.entries {
        PhysicalSegmentMembershipBlock::leaf(
            projection.tree_identity,
            projection.generation,
            projection.block_identity,
            entries.to_vec(),
            capacity,
        )
    } else {
        PhysicalSegmentMembershipBlock::branch(
            projection.tree_identity,
            projection.generation,
            projection.block_identity,
            projection.level,
            projection.children.unwrap_or_default().to_vec(),
            capacity,
        )
    };
    block.ok_or_else(|| RecoveryIntegrityIngressRejection::NonCanonicalEncoding.into())
}
