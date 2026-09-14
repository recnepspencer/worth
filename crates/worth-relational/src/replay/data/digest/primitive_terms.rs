use sha2::{Digest, Sha256};

use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, StructAspectValue};

use crate::history::data::{BranchId, CommitId, OrderedParentList};
use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::publication::patch::data::{
    PatchDetail, PatchOrdering, PatchPublicationMode, PatchStreamPosition,
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
    PublishedAuthoritativePatchOperation, PublishedAuthoritativePatchValue, RecordStructuralChange,
};
use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, SchemaBoundaryFingerprint,
    SchemaVersionId,
};
use crate::transactions::data::RecordRef;

#[path = "primitive_terms/diagnostic_value_terms.rs"]
mod diagnostic_value_terms;

pub(super) struct ReplayDigestBuilder {
    bytes: Vec<u8>,
}

impl ReplayDigestBuilder {
    pub(super) fn new(domain: &'static str) -> Self {
        Self { bytes: Vec::new() }.string(domain)
    }

    pub(super) fn tag(mut self, tag: u8) -> Self {
        self.bytes.push(tag);
        self
    }

    pub(super) fn bool(mut self, value: bool) -> Self {
        self.bytes.push(u8::from(value));
        self
    }

    pub(super) fn u32(mut self, value: u32) -> Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub(super) fn u64(mut self, value: u64) -> Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub(super) fn i64(mut self, value: i64) -> Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub(super) fn usize(self, value: usize) -> Self {
        self.u64(value as u64)
    }

    pub(super) fn string(mut self, value: &str) -> Self {
        self.bytes
            .extend_from_slice(&(value.len() as u64).to_le_bytes());
        self.bytes.extend_from_slice(value.as_bytes());
        self
    }

    pub(super) fn digest_bytes(mut self, value: &[u8; 32]) -> Self {
        self.bytes.extend_from_slice(value);
        self
    }

    pub(super) fn optional_digest(self, value: Option<&[u8; 32]>) -> Self {
        match value {
            Some(value) => self.tag(1).digest_bytes(value),
            None => self.tag(0),
        }
    }

    pub(super) fn branch_id(self, value: &BranchId) -> Self {
        self.string(&value.0)
    }

    pub(super) fn commit_id(self, value: CommitId) -> Self {
        self.u64(value.0)
    }

    pub(super) fn optional_commit_id(self, value: Option<CommitId>) -> Self {
        match value {
            Some(value) => self.tag(1).commit_id(value),
            None => self.tag(0),
        }
    }

    pub(super) fn version_id(self, value: VersionId) -> Self {
        self.u64(value.as_u64())
    }

    pub(super) fn optional_version_id(self, value: Option<VersionId>) -> Self {
        match value {
            Some(value) => self.tag(1).version_id(value),
            None => self.tag(0),
        }
    }

    pub(super) fn schema_version_id(self, value: SchemaVersionId) -> Self {
        self.u32(value.0)
    }

    pub(super) fn descriptor_semantics_version(self, value: DescriptorSemanticsVersion) -> Self {
        self.u32(value.0)
    }

    pub(super) fn descriptor_canonical_basis_version(
        self,
        value: DescriptorCanonicalBasisVersion,
    ) -> Self {
        self.u32(value.0)
    }

    pub(super) fn boundary_fingerprint(self, value: SchemaBoundaryFingerprint) -> Self {
        self.digest_bytes(&value.0)
    }

    pub(super) fn aspect_key(self, value: &AspectKey) -> Self {
        self.string(value.as_str())
    }

    pub(super) fn field_key(self, value: &FieldKey) -> Self {
        self.string(value.as_str())
    }

    pub(super) fn ordered_parent_list(mut self, parents: &OrderedParentList) -> Self {
        self = self.usize(parents.len());
        for parent in parents.as_slice() {
            self = self.commit_id(*parent);
        }
        self
    }

    pub(super) fn commit_id_sequence(mut self, values: &[CommitId]) -> Self {
        self = self.usize(values.len());
        for value in values {
            self = self.commit_id(*value);
        }
        self
    }

    pub(super) fn branch_id_sequence(mut self, values: &[BranchId]) -> Self {
        self = self.usize(values.len());
        for value in values {
            self = self.branch_id(value);
        }
        self
    }

    pub(super) fn record_ref(self, value: &RecordRef) -> Self {
        match value {
            RecordRef::Entity(entity_id) => self.tag(1).entity_id(*entity_id),
            RecordRef::Relation(relation_id) => self.tag(2).relation_id(*relation_id),
        }
    }

    pub(super) fn entity_id(self, value: EntityId) -> Self {
        self.u32(value.partition_value())
            .u64(value.local_slot_value())
            .u32(value.generation_value())
    }

    pub(super) fn relation_id(self, value: RelationId) -> Self {
        self.u32(value.partition_value())
            .u64(value.local_slot_value())
            .u32(value.generation_value())
    }

    pub(super) fn patch_ordering(self, value: PatchOrdering) -> Self {
        match value {
            PatchOrdering::CanonicalCommitOrder => self.tag(1),
        }
    }

    pub(super) fn patch_publication_mode(self, value: PatchPublicationMode) -> Self {
        match value {
            PatchPublicationMode::CommitNative => self.tag(1),
        }
    }

    pub(super) fn patch_stream_position(self, value: PatchStreamPosition) -> Self {
        self.u64(value.0)
    }

    #[cfg(test)]
    pub(super) fn optional_patch_stream_position(self, value: Option<PatchStreamPosition>) -> Self {
        match value {
            Some(value) => self.tag(1).patch_stream_position(value),
            None => self.tag(0),
        }
    }

    pub(super) fn structural_change(self, value: RecordStructuralChange) -> Self {
        match value {
            RecordStructuralChange::Created => self.tag(1),
            RecordStructuralChange::Updated => self.tag(2),
            RecordStructuralChange::Deleted => self.tag(3),
            RecordStructuralChange::RetainedForAudit => self.tag(4),
            RecordStructuralChange::MaterializationSuspended => self.tag(5),
            RecordStructuralChange::Rematerialized => self.tag(6),
        }
    }

    pub(super) fn patch_detail(mut self, value: &PatchDetail) -> Self {
        match value {
            PatchDetail::DenseBitset(bits) => {
                self = self.tag(1).usize(bits.len());
                for bit in bits {
                    self = self.u64(*bit);
                }
                self
            }
        }
    }

    pub(super) fn published_patch(mut self, value: &PublishedAuthoritativePatch) -> Self {
        let canonical = value.canonicalized();
        self = self.usize(canonical.full_grammar_operation_count());
        for operation in canonical.full_grammar_operations() {
            self = self.published_patch_operation(operation);
        }
        self
    }

    pub(super) fn published_patch_operation(
        self,
        operation: &PublishedAuthoritativePatchOperation,
    ) -> Self {
        match operation {
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                value,
            } => self
                .tag(1)
                .aspect_key(aspect_key)
                .u64(aspect_identity.0)
                .u64(contract_revision.0)
                .string(&binding.canonical_name())
                .published_patch_value(value),
            PublishedAuthoritativePatchOperation::WholeAspectClear {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
            } => self
                .tag(2)
                .aspect_key(aspect_key)
                .u64(aspect_identity.0)
                .u64(contract_revision.0)
                .string(&binding.canonical_name()),
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                field_sets,
                field_clears,
            } => self
                .tag(3)
                .aspect_key(aspect_key)
                .u64(aspect_identity.0)
                .u64(contract_revision.0)
                .string(&binding.canonical_name())
                .field_sets(field_sets)
                .field_clears(field_clears),
        }
    }

    fn published_patch_value(self, value: &PublishedAuthoritativePatchValue) -> Self {
        match value {
            PublishedAuthoritativePatchValue::Scalar(value) => self.tag(1).aspect_value(value),
            PublishedAuthoritativePatchValue::Struct(value) => self.tag(2).struct_value(value),
        }
    }

    fn field_sets(mut self, field_sets: &[PublishedAuthoritativeFieldSet]) -> Self {
        let mut canonical = field_sets.to_vec();
        canonical.sort();
        self = self.usize(canonical.len());
        for field_set in canonical {
            self = self
                .field_key(&field_set.field)
                .aspect_value(&field_set.value);
        }
        self
    }

    fn field_clears(mut self, field_clears: &[FieldKey]) -> Self {
        let mut canonical = field_clears.to_vec();
        canonical.sort();
        self = self.usize(canonical.len());
        for field_clear in canonical {
            self = self.field_key(&field_clear);
        }
        self
    }

    pub(super) fn struct_value(mut self, value: &StructAspectValue) -> Self {
        let fields = value.fields().collect::<Vec<_>>();
        self = self.usize(fields.len());
        for (field, value) in fields {
            self = self.field_key(field).aspect_value(value);
        }
        self
    }

    pub(super) fn aspect_value(self, value: &AspectValue) -> Self {
        self.tag(1)
            .byte_vec(&crate::aspect_wire::encode_aspect_value(value))
    }

    pub(super) fn byte_vec(mut self, bytes: &[u8]) -> Self {
        self.bytes
            .extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        self.bytes.extend_from_slice(bytes);
        self
    }

    pub(super) fn label(self, value: impl std::fmt::Debug) -> Self {
        self.string(&format!("{value:?}"))
    }

    pub(super) fn finish(self) -> [u8; 32] {
        Sha256::digest(self.bytes).into()
    }
}
