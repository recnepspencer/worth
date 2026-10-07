//! Sealed exact-media route view without a decoded placement allocation.
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PhysicalRecordFormatDeclaration,
    PhysicalRootRoutingBlockView,
};

use super::super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};
use super::validated_record;

#[derive(Debug)]
pub struct IntegrityValidatedRootRoutingBlockView<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    block: PhysicalRootRoutingBlockView<'media>,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedRootRoutingBlockView<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        block: PhysicalRootRoutingBlockView<'media>,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let validation_record = validated_record(
            scope,
            record_format,
            block.tree_identity(),
            block.reference(validated_range_checksum),
            validated_range_checksum,
            inspected,
        )?;
        Some(Self {
            scope,
            record_format,
            block,
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }
    pub const fn tree_identity(&self) -> u64 {
        self.block.tree_identity()
    }
    pub const fn generation(&self) -> u64 {
        self.block.generation()
    }
    pub const fn block_identity(&self) -> u64 {
        self.block.block()
    }
    pub const fn level(&self) -> u16 {
        self.block.level()
    }
    pub const fn count(&self) -> usize {
        self.block.count()
    }
    pub fn entries(
        &self,
    ) -> Option<
        impl ExactSizeIterator<Item = CurrentPhysicalRecordPlacement> + DoubleEndedIterator + '_,
    > {
        self.block.entries()
    }
    pub fn children(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = ManifestBlockReference> + DoubleEndedIterator + '_>
    {
        self.block.children()
    }
    pub fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }
    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
