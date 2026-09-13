use worth_store_physical_format::{
    FreeSpaceBlockReference, FreeSpaceMembershipBlockScopeIdentity,
    PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry,
};
use worth_store_physical_integrity::IntegrityValidatedFreeSpaceMembershipBlock;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedFreeSpaceMembershipBlock<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedFreeSpaceMembershipBlock<'media>,
}

pub(crate) struct FreeSpaceMembershipBlockProjection<'view> {
    pub identity: FreeSpaceMembershipBlockScopeIdentity,
    pub reference: FreeSpaceBlockReference,
    pub record_format: PhysicalRecordFormatDeclaration,
    pub generation: u64,
    pub block_identity: u64,
    pub level: u16,
    pub entries: Option<&'view [RecordFreeSpaceManifestEntry]>,
    pub children: Option<&'view [FreeSpaceBlockReference]>,
}

impl<'media> IntegrityAdmittedFreeSpaceMembershipBlock<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedFreeSpaceMembershipBlock<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project<'view>(
        &'view self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> FreeSpaceMembershipBlockProjection<'view> {
        counters.record_owner_projection();
        FreeSpaceMembershipBlockProjection {
            identity: self.validated.identity(),
            reference: self.validated.reference(),
            record_format: self.validated.record_format(),
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
        validated: IntegrityValidatedFreeSpaceMembershipBlock<'media>,
    ) {
        let _ = IntegrityAdmittedFreeSpaceMembershipBlock::bind(source, validated);
    }
    let _ = bind;
}
