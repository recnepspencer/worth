use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, ManifestBlockReference, PhysicalRecordFormatDeclaration,
    PhysicalRootRoutingBlock, PhysicalTreeIdentity, RootRoutingBlockScopeIdentity,
};
use worth_store_physical_integrity::{validate_root_routing_block, PhysicalArtifactScope};

use super::{expected_range, source_input, source_range};
use crate::integrity_ingress::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

pub(crate) struct AdmittedRootRoutingProjection {
    pub(crate) block: PhysicalRootRoutingBlock,
    pub(crate) page_facts: worth_store_recovery_physics::PhysicalManifestBlockProjection,
}

pub(crate) fn root_routing_block(
    source: &ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    tree: PhysicalTreeIdentity,
    reference: ManifestBlockReference,
    capacity: u16,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<AdmittedRootRoutingProjection, RecoveryIntegrityIngressRejection> {
    let identity = RootRoutingBlockScopeIdentity::new(tree, reference);
    let expected_scope =
        PhysicalArtifactScope::root_routing_block(store, format, identity, expected_range(format));
    let range =
        source_range(source).map_err(|rejection| trace.reject(expected_scope, rejection))?;
    let scope = PhysicalArtifactScope::root_routing_block(store, format, identity, range);
    let validation = validate_root_routing_block(source_input(source)?, scope).0;
    let attempt = IntegrityAdmittedRecoveryArtifact::bind_root_routing_block(
        source,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    let admitted = attempt.into_outcome()?;
    let IntegrityAdmittedRecoveryArtifact::RootRoutingBlock(admitted) = admitted else {
        unreachable!("root-routing admission preserves its family")
    };
    let projection = admitted.project(trace.counters_mut());
    let block = if let Some(entries) = projection.entries {
        PhysicalRootRoutingBlock::leaf(
            projection.tree_identity,
            projection.generation,
            projection.block_identity,
            entries.to_vec(),
            capacity,
        )
    } else {
        PhysicalRootRoutingBlock::branch(
            projection.tree_identity,
            projection.generation,
            projection.block_identity,
            projection.level,
            projection.children.unwrap_or_default().to_vec(),
            capacity,
        )
    };
    let block = block.ok_or(RecoveryIntegrityIngressRejection::NonCanonicalEncoding)?;
    let page_facts =
        worth_store_recovery_physics::PhysicalManifestBlockProjection::from_projected_block(
            reference,
            block.clone(),
        );
    Ok(AdmittedRootRoutingProjection { block, page_facts })
}
