use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PhysicalRecordFormatDeclaration,
};
use worth_store_physical_integrity::IntegrityValidatedRootRoutingBlock;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedRootRoutingBlock<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedRootRoutingBlock<'media>,
}

pub(crate) struct RootRoutingBlockProjection<'view> {
    pub record_format: PhysicalRecordFormatDeclaration,
    pub tree_identity: u64,
    pub generation: u64,
    pub block_identity: u64,
    pub level: u16,
    pub entries: Option<&'view [CurrentPhysicalRecordPlacement]>,
    pub children: Option<&'view [ManifestBlockReference]>,
}

impl<'media> IntegrityAdmittedRootRoutingBlock<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedRootRoutingBlock<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project<'view>(
        &'view self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> RootRoutingBlockProjection<'view> {
        counters.record_owner_projection();
        RootRoutingBlockProjection {
            record_format: self.validated.record_format(),
            tree_identity: self.validated.tree_identity(),
            generation: self.validated.generation(),
            block_identity: self.validated.block_identity(),
            level: self.validated.level(),
            entries: self.validated.entries(),
            children: self.validated.children(),
        }
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedRootRoutingBlock<'media>,
    ) {
        let _ = IntegrityAdmittedRootRoutingBlock::bind(source, validated);
    }
    let _ = bind;
}
