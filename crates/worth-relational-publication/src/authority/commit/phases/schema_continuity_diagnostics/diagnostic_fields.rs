use worth_relational_contracts::internal_port::diagnostics::data::fields::{
    RelationalDiagnosticFields, RelationalDiagnosticValue,
};

use super::diagnostic_value_terms::{
    boundary_fingerprint, branch_id, contract_field_path, count,
    descriptor_canonical_basis_version, descriptor_semantics_version, fields, label,
    optional_branch_id, optional_descriptor_semantics_version, optional_kind_id,
    optional_schema_version_id, schema_id, schema_ids, schema_version_id, schema_version_ids,
    strata, string_array,
};
use super::field_shapes::{
    SchemaBridgeDescriptorFields, SchemaContinuityFailureFields, SchemaDescriptorVersionFields,
    SchemaDiffAtomTraceFields, SchemaDiffDetailFields, SchemaInterpretationFields,
    SchemaLineageFields, SchemaReconciliationFields, SchemaTransitionClassificationFields,
    SchemaTransitionRejectedFields, SchemaTransitionSummaryFields,
};

pub(super) trait IntoSchemaContinuityDiagnosticFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields;
}

pub(super) fn diagnostics_fields(
    fields: impl IntoSchemaContinuityDiagnosticFields,
) -> RelationalDiagnosticFields {
    fields.into_diagnostic_fields()
}

impl IntoSchemaContinuityDiagnosticFields for SchemaTransitionSummaryFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("branch_id", branch_id(self.branch_id)),
            ("source_schema_id", schema_id(self.source_schema_id)),
            (
                "source_schema_version_id",
                schema_version_id(self.source_schema_version_id),
            ),
            ("target_schema_id", schema_id(self.target_schema_id)),
            (
                "target_schema_version_id",
                schema_version_id(self.target_schema_version_id),
            ),
            ("changed_atom_count", count(self.changed_atom_count)),
            ("changed_strata", strata(self.changed_strata)),
            (
                "historical_interpretation",
                label(self.historical_interpretation),
            ),
            ("continuation", label(self.continuation)),
            ("bridgeability", label(self.bridgeability)),
            ("reconciliation", label(self.reconciliation)),
            (
                "descriptor_semantics_version",
                descriptor_semantics_version(self.descriptor_semantics_version),
            ),
            (
                "descriptor_canonical_basis_version",
                descriptor_canonical_basis_version(self.descriptor_canonical_basis_version),
            ),
            (
                "normalized_boundary_count",
                count(self.normalized_boundary_count),
            ),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaContinuityFailureFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("branch_id", branch_id(self.branch_id)),
            ("conflict_class", label(self.conflict_class)),
            ("detail", label(self.detail)),
            (
                "previous_schema_version",
                optional_schema_version_id(self.previous_schema_version),
            ),
            (
                "previous_descriptor_semantics_version",
                optional_descriptor_semantics_version(self.previous_descriptor_semantics_version),
            ),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaTransitionRejectedFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("source_schema_id", schema_id(self.source_schema_id)),
            (
                "source_schema_version_id",
                schema_version_id(self.source_schema_version_id),
            ),
            ("target_schema_id", schema_id(self.target_schema_id)),
            (
                "target_schema_version_id",
                schema_version_id(self.target_schema_version_id),
            ),
            ("changed_atom_count", count(self.changed_atom_count)),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaDiffAtomTraceFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("diff_atom_index", count(self.diff_atom_index)),
            ("element_kind", label(self.element_kind)),
            ("schema_id", schema_id(self.schema_id)),
            (
                "schema_version_id",
                schema_version_id(self.schema_version_id),
            ),
            ("kind_id", optional_kind_id(self.kind_id)),
            ("element_name", label(self.element_name)),
            ("strata", strata(self.strata)),
            ("publication_impact", label(self.publication_impact)),
            ("subscriber_impact", label(self.subscriber_impact)),
            (
                "historical_interpretation",
                label(self.historical_interpretation),
            ),
            ("detail", schema_diff_detail(self.detail)),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaBridgeDescriptorFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            (
                "boundary_fingerprint",
                boundary_fingerprint(self.boundary_fingerprint),
            ),
            ("continuation", label(self.continuation)),
            ("bridgeability", label(self.bridgeability)),
            (
                "normalized_boundary_count",
                count(self.normalized_boundary_count),
            ),
            (
                "descriptor_canonical_basis_version",
                descriptor_canonical_basis_version(self.descriptor_canonical_basis_version),
            ),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaInterpretationFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            (
                "boundary_fingerprint",
                boundary_fingerprint(self.boundary_fingerprint),
            ),
            (
                "historical_interpretation",
                label(self.historical_interpretation),
            ),
            ("changed_strata", strata(self.changed_strata)),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaReconciliationFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("classification", label(self.classification)),
            ("policy", label(self.policy)),
            ("resulting_schema_id", schema_id(self.resulting_schema_id)),
            (
                "resulting_schema_version_id",
                schema_version_id(self.resulting_schema_version_id),
            ),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaDescriptorVersionFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            (
                "descriptor_semantics_version",
                descriptor_semantics_version(self.descriptor_semantics_version),
            ),
            (
                "continuation_canonical_basis_version",
                descriptor_canonical_basis_version(self.continuation_canonical_basis_version),
            ),
            (
                "reconciliation_canonical_basis_version",
                descriptor_canonical_basis_version(self.reconciliation_canonical_basis_version),
            ),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaTransitionClassificationFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("branch_id", branch_id(self.branch_id)),
            (
                "boundary_fingerprint",
                boundary_fingerprint(self.boundary_fingerprint),
            ),
            ("continuation", label(self.continuation)),
            ("bridgeability", label(self.bridgeability)),
            (
                "historical_interpretation",
                label(self.historical_interpretation),
            ),
            ("changed_strata", strata(self.changed_strata)),
            ("reconciliation", label(self.reconciliation)),
            ("policy", label(self.policy)),
        ])
    }
}

impl IntoSchemaContinuityDiagnosticFields for SchemaLineageFields {
    fn into_diagnostic_fields(self) -> RelationalDiagnosticFields {
        fields([
            ("resulting_schema_id", schema_id(self.resulting_schema_id)),
            (
                "resulting_schema_version_id",
                schema_version_id(self.resulting_schema_version_id),
            ),
            ("parent_schema_ids", schema_ids(self.parent_schema_ids)),
            (
                "parent_schema_version_ids",
                schema_version_ids(self.parent_schema_version_ids),
            ),
            ("ordering_mode", label(self.ordering_mode)),
            ("ordering_semantics", label(self.ordering_semantics)),
            ("branch_context", optional_branch_id(self.branch_context)),
        ])
    }
}

fn schema_diff_detail(detail: SchemaDiffDetailFields) -> RelationalDiagnosticValue {
    match detail {
        SchemaDiffDetailFields::AddedField {
            field,
            required,
            default_expression,
        } => RelationalDiagnosticValue::object([
            ("kind", label("AddedField")),
            ("field_path", contract_field_path(field)),
            ("required", RelationalDiagnosticValue::Bool(required)),
            (
                "default_expression",
                RelationalDiagnosticValue::optional(default_expression.map(label)),
            ),
        ]),
        SchemaDiffDetailFields::RemovedField { field } => RelationalDiagnosticValue::object([
            ("kind", label("RemovedField")),
            ("field_path", contract_field_path(field)),
        ]),
        SchemaDiffDetailFields::TypeChanged {
            field,
            from_type,
            to_type,
        } => RelationalDiagnosticValue::object([
            ("kind", label("TypeChanged")),
            ("field_path", contract_field_path(field)),
            ("from_type", label(from_type)),
            ("to_type", label(to_type)),
        ]),
        SchemaDiffDetailFields::EnumDomainExpanded {
            field,
            added_variants,
        } => RelationalDiagnosticValue::object([
            ("kind", label("EnumDomainExpanded")),
            ("field_path", contract_field_path(field)),
            ("added_variants", string_array(added_variants)),
        ]),
        SchemaDiffDetailFields::InvariantContractChanged { contract_name } => {
            RelationalDiagnosticValue::object([
                ("kind", label("InvariantContractChanged")),
                ("contract_name", label(contract_name)),
            ])
        }
        SchemaDiffDetailFields::ProjectionContractChanged { projection_name } => {
            RelationalDiagnosticValue::object([
                ("kind", label("ProjectionContractChanged")),
                ("projection_name", label(projection_name)),
            ])
        }
        SchemaDiffDetailFields::SubscriberContractChanged { contract_name } => {
            RelationalDiagnosticValue::object([
                ("kind", label("SubscriberContractChanged")),
                ("contract_name", label(contract_name)),
            ])
        }
        SchemaDiffDetailFields::FreeText {
            detail,
            declared_intent,
        } => RelationalDiagnosticValue::object([
            ("kind", label("FreeText")),
            ("detail", label(detail)),
            ("declared_intent", label(format!("{declared_intent:?}"))),
        ]),
    }
}
