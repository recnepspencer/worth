use worth_foundational::{CanonicalFieldPath, FieldKey};
use worth_relational_contracts::internal_port as contracts;

use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use contracts::history::data::coordinates::BranchId;
use contracts::identity::data::primitives::KindId;
use contracts::schema::data::continuity::boundary_fingerprint::SchemaBoundaryFingerprint;
use contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion;
use contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;
use contracts::schema::data::continuity::transition_classification::SchemaStratum;
use contracts::schema::data::SchemaId;
use contracts::schema::data::SchemaVersionId;

pub(super) fn fields(
    entries: impl IntoIterator<Item = (&'static str, RelationalDiagnosticValue)>,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object(entries).into()
}

pub(super) fn branch_id(value: BranchId) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::BranchId(value)
}

pub(super) fn optional_branch_id(value: Option<BranchId>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(value.map(branch_id))
}

pub(super) fn schema_id(value: SchemaId) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::SchemaId(value)
}

pub(super) fn schema_ids(values: Vec<SchemaId>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(values.into_iter().map(schema_id))
}

pub(super) fn schema_version_id(value: SchemaVersionId) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::SchemaVersionId(value)
}

pub(super) fn schema_version_ids(values: Vec<SchemaVersionId>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(values.into_iter().map(schema_version_id))
}

pub(super) fn optional_schema_version_id(
    value: Option<SchemaVersionId>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(value.map(schema_version_id))
}

pub(super) fn descriptor_semantics_version(
    value: DescriptorSemanticsVersion,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::DescriptorSemanticsVersion(value)
}

pub(super) fn optional_descriptor_semantics_version(
    value: Option<DescriptorSemanticsVersion>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(value.map(descriptor_semantics_version))
}

pub(super) fn descriptor_canonical_basis_version(
    value: DescriptorCanonicalBasisVersion,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::DescriptorCanonicalBasisVersion(value)
}

pub(super) fn boundary_fingerprint(value: SchemaBoundaryFingerprint) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::SchemaBoundaryFingerprint(value)
}

pub(super) fn optional_kind_id(value: Option<KindId>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(value.map(RelationalDiagnosticValue::KindId))
}

pub(super) fn contract_field_path(field: FieldKey) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::FieldPath(CanonicalFieldPath::single(field))
}

pub(super) fn strata(values: Vec<SchemaStratum>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        values
            .into_iter()
            .map(|stratum| label(format!("{stratum:?}"))),
    )
}

pub(super) fn string_array(values: Vec<String>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(values.into_iter().map(label))
}

pub(super) fn label(value: impl Into<String>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::string(value)
}

pub(super) fn count(value: usize) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::unsigned(value)
}
