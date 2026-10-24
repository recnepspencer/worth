use worth_foundational::facade::{AspectFieldLocator, CanonicalBasisReadyArtifact};

use crate::canonical_basis_ready_sequence::canonical_basis_ready_sequence;

use super::ReplayDigestBuilder;
use crate::diagnostics::data::RelationalDiagnosticValue;

impl ReplayDigestBuilder {
    pub(in crate::replay::data::digest) fn diagnostic_value(
        mut self,
        value: &RelationalDiagnosticValue,
    ) -> Self {
        match value {
            RelationalDiagnosticValue::Null => self.tag(0),
            RelationalDiagnosticValue::Bool(value) => self.tag(1).bool(*value),
            RelationalDiagnosticValue::Unsigned(value) => self.tag(2).u64(*value),
            RelationalDiagnosticValue::Signed(value) => self.tag(3).i64(*value),
            RelationalDiagnosticValue::String(value) => self.tag(4).string(value),
            RelationalDiagnosticValue::Array(values) => {
                self = self.tag(5).usize(values.len());
                for value in values {
                    self = self.diagnostic_value(value);
                }
                self
            }
            RelationalDiagnosticValue::Object(fields) => {
                self = self.tag(6).usize(fields.len());
                for (key, value) in fields {
                    self = self.string(key).diagnostic_value(value);
                }
                self
            }
            RelationalDiagnosticValue::AspectKey(value) => self.tag(7).aspect_key(value),
            RelationalDiagnosticValue::FieldKey(value) => self.tag(8).field_key(value),
            RelationalDiagnosticValue::FieldPath(fields) => {
                self = self.tag(9).usize(fields.fields().len());
                for field in fields.fields() {
                    self = self.field_key(field);
                }
                self
            }
            RelationalDiagnosticValue::AspectValue(value) => self.tag(10).aspect_value(value),
            RelationalDiagnosticValue::StructAspectValue(value) => self.tag(11).struct_value(value),
            RelationalDiagnosticValue::PartitionId(value) => self.tag(12).u32(value.as_u32()),
            RelationalDiagnosticValue::KindId(value) => self.tag(13).u32(value.as_u32()),
            RelationalDiagnosticValue::VersionId(value) => self.tag(14).version_id(*value),
            RelationalDiagnosticValue::LineageId(value) => self.tag(15).u64(value.as_u64()),
            RelationalDiagnosticValue::CommitId(value) => self.tag(16).commit_id(*value),
            RelationalDiagnosticValue::BranchId(value) => self.tag(17).branch_id(value),
            RelationalDiagnosticValue::PatchStreamPosition(value) => {
                self.tag(18).patch_stream_position(*value)
            }
            RelationalDiagnosticValue::SchemaVersionId(value) => {
                self.tag(19).schema_version_id(*value)
            }
            RelationalDiagnosticValue::DescriptorSemanticsVersion(value) => {
                self.tag(20).descriptor_semantics_version(*value)
            }
            RelationalDiagnosticValue::DescriptorCanonicalBasisVersion(value) => {
                self.tag(21).descriptor_canonical_basis_version(*value)
            }
            RelationalDiagnosticValue::EntityId(value) => self.tag(22).entity_id(*value),
            RelationalDiagnosticValue::RelationId(value) => self.tag(23).relation_id(*value),
            RelationalDiagnosticValue::SchemaBoundaryFingerprint(value) => {
                self.tag(24).boundary_fingerprint(*value)
            }
            RelationalDiagnosticValue::AspectValueLocator(value) => self.tag(25).label(value),
            RelationalDiagnosticValue::DiagnosticMask(value) => self.tag(26).label(value),
            RelationalDiagnosticValue::SnapshotId(value) => self.tag(27).label(value),
            // Tags 28 and 29 named retired filesystem checkpoint and segment
            // identifiers; they are never reassigned.
            RelationalDiagnosticValue::DerivedIndexId(value) => self.tag(30).label(value),
            RelationalDiagnosticValue::DerivedIndexGenerationId(value) => self.tag(31).label(value),
            RelationalDiagnosticValue::CorrespondenceCandidateId(value) => self.tag(32).u64(*value),
            RelationalDiagnosticValue::ReplaySchemaVersion(value) => self.tag(33).label(value),
            RelationalDiagnosticValue::SchemaId(value) => self.tag(34).label(value),
            RelationalDiagnosticValue::ContractId(value) => self.tag(35).label(value),
            RelationalDiagnosticValue::DiagnosticMaskLocator(value) => self.tag(36).label(value),
            RelationalDiagnosticValue::CanonicalBasis(value) => {
                self.tag(37).canonical_basis_ready(value)
            }
            RelationalDiagnosticValue::CanonicalBytes(value) => self.tag(38).byte_vec(value),
            RelationalDiagnosticValue::AspectFieldLocator(value) => {
                self.tag(39).aspect_field_locator(value)
            }
        }
    }

    fn aspect_field_locator(self, value: &AspectFieldLocator) -> Self {
        self.byte_vec(&crate::aspect_wire::encode_aspect_field_locator(value))
    }

    fn canonical_basis_ready(mut self, value: &CanonicalBasisReadyArtifact) -> Self {
        let ready_sequence = canonical_basis_ready_sequence(value);
        self = self
            .label(ready_sequence.domain())
            .string(ready_sequence.version().as_str())
            .usize(ready_sequence.entries().len());
        for entry in ready_sequence.entries() {
            self = self
                .label((entry.domain(), entry.locus()))
                .label(entry.kind())
                .label(entry.value());
        }
        self
    }
}
