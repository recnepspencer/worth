use super::PhysicalCurrentRootOwner;
use crate::physical_runtime::durability::{
    NamespaceDurableRetirementRoot, PhysicalRetirementDenial, RetainedPhysicalRoot,
};
use sha2::{Digest, Sha256};

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn advance_retirement_root(
        &self,
        durable: NamespaceDurableRetirementRoot,
        format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut state = self.lock_publication_state();
        let identity = durable.transition.identity();
        let Some((operation, release, wal_digest)) = identity.retirement_basis() else {
            return Err(PhysicalRetirementDenial::Delete);
        };
        let candidate = durable.candidate.successor_root();
        let digest: [u8; 32] = Sha256::digest(candidate.encode(format)).into();
        if durable.transition.source_root() != &state.current_root
            || durable.candidate.source_root() != &state.current_root
            || release.source_generation() != state.current_root.generation()
            || release.candidate_generation() != candidate.generation()
            || release.candidate_digest() != digest
            || wal_digest != durable.receipt.payload_digest()
        {
            return Err(PhysicalRetirementDenial::Delete);
        }
        let (source, free, root, _, _) = durable.candidate.into_root_parts();
        state.namespace_evidence = crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::RetirementCurrentRoot {
            operation, source_generation: source.generation(), current_generation: root.generation(),
            replacement: durable.replacement, namespace_synchronization: durable.namespace,
        };
        state.previous_root = Some(RetainedPhysicalRoot::from_manifest(source));
        state.current_root = root;
        state.free_space = free;
        durable.transition.release();
        Ok(())
    }
}
