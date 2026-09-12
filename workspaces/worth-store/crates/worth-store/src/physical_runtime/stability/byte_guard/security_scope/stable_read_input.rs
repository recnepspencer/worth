use worth_store_physical_format::{
    PhysicalCellReuseDomain, PhysicalDecodedHeader, PhysicalGeneration,
    PhysicalHeaderDecodeWitness, PhysicalPageHeader, PhysicalSecurityMetadataEnvelope,
    SegmentPageSecurityMetadataEnvelope, SlotGenerationCell,
};
use worth_store_security::StoreSecurityMetadata;

use crate::physical_runtime::stability::PhysicalByteGuardScope;
use worth_store_physical_isolation::{
    CurrentPhysicalRoot, PhysicalReadProtectedFootprintBasis, StablePhysicalReadHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadSecurityScopeCarrierBasis {
    page_header: PhysicalPageHeader,
    page_decode: PhysicalHeaderDecodeWitness,
    manifest_page_slot: SlotGenerationCell,
    guard_scope: PhysicalByteGuardScope,
}

impl StableReadSecurityScopeCarrierBasis {
    pub fn new(
        guard_scope: PhysicalByteGuardScope,
        page_header: &PhysicalSecurityMetadataEnvelope<PhysicalPageHeader, StoreSecurityMetadata>,
        page_decode: PhysicalHeaderDecodeWitness,
        manifest_entry: &SegmentPageSecurityMetadataEnvelope<StoreSecurityMetadata>,
    ) -> Self {
        Self {
            page_header: page_header.header(),
            page_decode,
            manifest_page_slot: manifest_entry.artifact().page_slot(),
            guard_scope,
        }
    }

    pub const fn page_header_generation(self) -> PhysicalGeneration {
        self.page_header.generation()
    }

    pub const fn manifest_page_slot(self) -> SlotGenerationCell {
        self.manifest_page_slot
    }

    pub const fn guard_scope(self) -> PhysicalByteGuardScope {
        self.guard_scope
    }

    pub fn matches_guard_scope(self, guard_scope: PhysicalByteGuardScope) -> bool {
        let owner = guard_scope.reference().owner();
        let page_owner = self.page_decode.owner();
        self.guard_scope == guard_scope
            && self.page_decode.header() == PhysicalDecodedHeader::Page(self.page_header)
            && page_owner.domain() == PhysicalCellReuseDomain::Page
            && page_owner.segment_id() == owner.segment_id()
            && page_owner.page_id() == owner.page_id()
            && page_owner.generation() == owner.generation()
            && self.manifest_page_slot.owner() == owner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadSecurityScopePropagationInput {
    protected_root: CurrentPhysicalRoot,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
    carrier_basis: StableReadSecurityScopeCarrierBasis,
    page_metadata: StoreSecurityMetadata,
    manifest_metadata: StoreSecurityMetadata,
}

impl StableReadSecurityScopePropagationInput {
    pub fn new(
        handle: &StablePhysicalReadHandle,
        guard_scope: PhysicalByteGuardScope,
        page_header: &PhysicalSecurityMetadataEnvelope<PhysicalPageHeader, StoreSecurityMetadata>,
        page_decode: PhysicalHeaderDecodeWitness,
        manifest_entry: &SegmentPageSecurityMetadataEnvelope<StoreSecurityMetadata>,
    ) -> Self {
        Self {
            protected_root: handle.plan().root(),
            footprint_basis: handle.plan().footprint().declared_footprint_basis(),
            carrier_basis: StableReadSecurityScopeCarrierBasis::new(
                guard_scope,
                page_header,
                page_decode,
                manifest_entry,
            ),
            page_metadata: page_header.security_metadata(),
            manifest_metadata: manifest_entry.security_metadata(),
        }
    }

    pub const fn protected_root(self) -> CurrentPhysicalRoot {
        self.protected_root
    }

    pub const fn footprint_basis(self) -> PhysicalReadProtectedFootprintBasis {
        self.footprint_basis
    }

    pub const fn guard_scope(self) -> PhysicalByteGuardScope {
        self.carrier_basis.guard_scope()
    }

    pub const fn carrier_basis(self) -> StableReadSecurityScopeCarrierBasis {
        self.carrier_basis
    }

    pub const fn page_metadata(self) -> StoreSecurityMetadata {
        self.page_metadata
    }

    pub const fn manifest_metadata(self) -> StoreSecurityMetadata {
        self.manifest_metadata
    }
}
