use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, RecordSegmentPageManifestEntry, SegmentManifestBlockReference,
};
use worth_store_physical_integrity::IntegrityValidatedSegmentMembershipBlock;

use super::super::admission::require_observed_recovery_source;
use super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedSegmentMembershipBlock<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedSegmentMembershipBlock<'media>,
}

pub(crate) struct SegmentMembershipBlockProjection<'view> {
    pub record_format: PhysicalRecordFormatDeclaration,
    pub tree_identity: u64,
    pub generation: u64,
    pub block_identity: u64,
    pub level: u16,
    pub entries: Option<&'view [RecordSegmentPageManifestEntry]>,
    pub children: Option<&'view [SegmentManifestBlockReference]>,
}

impl<'media> IntegrityAdmittedSegmentMembershipBlock<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedSegmentMembershipBlock<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project<'view>(
        &'view self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> SegmentMembershipBlockProjection<'view> {
        counters.record_owner_projection();
        SegmentMembershipBlockProjection {
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
        validated: IntegrityValidatedSegmentMembershipBlock<'media>,
    ) {
        let _ = IntegrityAdmittedSegmentMembershipBlock::bind(source, validated);
    }
    let _ = bind;
}
