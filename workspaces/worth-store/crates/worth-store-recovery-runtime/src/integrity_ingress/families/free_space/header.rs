use worth_store_physical_format::{
    DurableArtifactCrc32c, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity,
    PhysicalRecordFormatDeclaration,
};
use worth_store_physical_integrity::IntegrityValidatedFreeSpaceHeader;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedFreeSpaceHeader<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedFreeSpaceHeader<'media>,
}

pub(crate) struct FreeSpaceHeaderProjection {
    pub identity: FreeSpaceHeaderScopeIdentity,
    pub record_format: PhysicalRecordFormatDeclaration,
    pub root: Option<FreeSpaceBlockReference>,
    pub complete_child_checksum: DurableArtifactCrc32c,
    pub node_capacity: u16,
    pub segment_page_capacity: u32,
    pub entry_count: u64,
    pub next_segment: u64,
    pub next_page: u64,
    pub next_extent: u64,
    pub next_block: u64,
}

impl<'media> IntegrityAdmittedFreeSpaceHeader<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedFreeSpaceHeader<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> FreeSpaceHeaderProjection {
        counters.record_owner_projection();
        FreeSpaceHeaderProjection {
            identity: self.validated.identity(),
            record_format: self.validated.record_format(),
            root: self.validated.root(),
            complete_child_checksum: self.validated.complete_child_checksum(),
            node_capacity: self.validated.node_capacity(),
            segment_page_capacity: self.validated.segment_page_capacity(),
            entry_count: self.validated.entry_count(),
            next_segment: self.validated.next_segment(),
            next_page: self.validated.next_page(),
            next_extent: self.validated.next_extent(),
            next_block: self.validated.next_block(),
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
        validated: IntegrityValidatedFreeSpaceHeader<'media>,
    ) {
        let _ = IntegrityAdmittedFreeSpaceHeader::bind(source, validated);
    }
    let _ = bind;
}
