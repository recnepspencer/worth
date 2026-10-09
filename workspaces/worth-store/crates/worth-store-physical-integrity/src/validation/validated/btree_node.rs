use worth_store_physical_format::{durable_artifact_checksum, BTreeNodeV1};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedBTreeNode<'media> {
    node: BTreeNodeV1,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedBTreeNode<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        node: BTreeNodeV1,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if scope.btree_node_identity().map(|(_, family)| family) != Some(node.family_code())
            || scope.byte_range().length() != inspected.byte_count()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.exact_btree_node_scope_digest()),
            PhysicalIntegrityValidationDigest::crc32c(durable_artifact_checksum(inspected.bytes())),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            node,
            validation_record,
            inspected,
        })
    }

    pub fn node(&self) -> &BTreeNodeV1 {
        &self.node
    }

    pub fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    /// Equal bytes in another allocation are not the inspected incarnation.
    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
