use crate::history::data::BranchId;
use crate::identity::data::KindId;
use worth_foundational::FieldKey;

use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, FreeFormSchemaDiffIntent,
    SchemaBoundaryFingerprint, SchemaDiffAtom, SchemaDiffDetail, SchemaId, SchemaStratum,
    SchemaVersionId,
};

#[derive(Debug)]
pub(super) struct SchemaTransitionSummaryFields {
    pub(super) branch_id: BranchId,
    pub(super) source_schema_id: SchemaId,
    pub(super) source_schema_version_id: SchemaVersionId,
    pub(super) target_schema_id: SchemaId,
    pub(super) target_schema_version_id: SchemaVersionId,
    pub(super) changed_atom_count: usize,
    pub(super) changed_strata: Vec<SchemaStratum>,
    pub(super) historical_interpretation: String,
    pub(super) continuation: String,
    pub(super) bridgeability: String,
    pub(super) reconciliation: String,
    pub(super) descriptor_semantics_version: DescriptorSemanticsVersion,
    pub(super) descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    pub(super) normalized_boundary_count: usize,
}

#[derive(Debug)]
pub(super) struct SchemaContinuityFailureFields {
    pub(super) branch_id: BranchId,
    pub(super) conflict_class: String,
    pub(super) detail: String,
    pub(super) previous_schema_version: Option<SchemaVersionId>,
    pub(super) previous_descriptor_semantics_version: Option<DescriptorSemanticsVersion>,
}

#[derive(Debug)]
pub(super) struct SchemaTransitionRejectedFields {
    pub(super) source_schema_id: SchemaId,
    pub(super) source_schema_version_id: SchemaVersionId,
    pub(super) target_schema_id: SchemaId,
    pub(super) target_schema_version_id: SchemaVersionId,
    pub(super) changed_atom_count: usize,
}

#[derive(Debug)]
pub(super) struct SchemaDiffAtomTraceFields {
    pub(super) diff_atom_index: usize,
    pub(super) element_kind: String,
    pub(super) schema_id: SchemaId,
    pub(super) schema_version_id: SchemaVersionId,
    pub(super) kind_id: Option<KindId>,
    pub(super) element_name: String,
    pub(super) strata: Vec<SchemaStratum>,
    pub(super) publication_impact: String,
    pub(super) subscriber_impact: String,
    pub(super) historical_interpretation: String,
    pub(super) detail: SchemaDiffDetailFields,
}

#[derive(Debug)]
pub(super) struct SchemaBridgeDescriptorFields {
    pub(super) boundary_fingerprint: SchemaBoundaryFingerprint,
    pub(super) continuation: String,
    pub(super) bridgeability: String,
    pub(super) normalized_boundary_count: usize,
    pub(super) descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
}

#[derive(Debug)]
pub(super) struct SchemaInterpretationFields {
    pub(super) boundary_fingerprint: SchemaBoundaryFingerprint,
    pub(super) historical_interpretation: String,
    pub(super) changed_strata: Vec<SchemaStratum>,
}

#[derive(Debug)]
pub(super) struct SchemaReconciliationFields {
    pub(super) classification: String,
    pub(super) policy: String,
    pub(super) resulting_schema_id: SchemaId,
    pub(super) resulting_schema_version_id: SchemaVersionId,
}

#[derive(Debug)]
pub(super) struct SchemaDescriptorVersionFields {
    pub(super) descriptor_semantics_version: DescriptorSemanticsVersion,
    pub(super) continuation_canonical_basis_version: DescriptorCanonicalBasisVersion,
    pub(super) reconciliation_canonical_basis_version: DescriptorCanonicalBasisVersion,
}

#[derive(Debug)]
pub(super) struct SchemaTransitionClassificationFields {
    pub(super) branch_id: BranchId,
    pub(super) boundary_fingerprint: SchemaBoundaryFingerprint,
    pub(super) continuation: String,
    pub(super) bridgeability: String,
    pub(super) historical_interpretation: String,
    pub(super) changed_strata: Vec<SchemaStratum>,
    pub(super) reconciliation: String,
    pub(super) policy: String,
}

#[derive(Debug)]
pub(super) struct SchemaLineageFields {
    pub(super) resulting_schema_id: SchemaId,
    pub(super) resulting_schema_version_id: SchemaVersionId,
    pub(super) parent_schema_ids: Vec<SchemaId>,
    pub(super) parent_schema_version_ids: Vec<SchemaVersionId>,
    pub(super) ordering_mode: String,
    pub(super) ordering_semantics: String,
    pub(super) branch_context: Option<BranchId>,
}

#[derive(Debug)]
pub(super) enum SchemaDiffDetailFields {
    AddedField {
        field: FieldKey,
        required: bool,
        default_expression: Option<String>,
    },
    RemovedField {
        field: FieldKey,
    },
    TypeChanged {
        field: FieldKey,
        from_type: String,
        to_type: String,
    },
    EnumDomainExpanded {
        field: FieldKey,
        added_variants: Vec<String>,
    },
    InvariantContractChanged {
        contract_name: String,
    },
    ProjectionContractChanged {
        projection_name: String,
    },
    SubscriberContractChanged {
        contract_name: String,
    },
    FreeText {
        detail: String,
        declared_intent: FreeFormSchemaDiffIntent,
    },
}

pub(super) fn schema_diff_atom_trace_fields(
    index: usize,
    atom: &SchemaDiffAtom,
) -> SchemaDiffAtomTraceFields {
    SchemaDiffAtomTraceFields {
        diff_atom_index: index,
        element_kind: format!("{:?}", atom.element.kind),
        schema_id: atom.element.schema_id.clone(),
        schema_version_id: atom.element.schema_version_id,
        kind_id: atom.element.kind_id,
        element_name: atom.element.element_name.to_string(),
        strata: atom.strata.clone(),
        publication_impact: format!("{:?}", atom.publication_impact),
        subscriber_impact: format!("{:?}", atom.subscriber_impact),
        historical_interpretation: format!("{:?}", atom.historical_interpretation),
        detail: schema_diff_detail_fields(&atom.detail),
    }
}

fn schema_diff_detail_fields(detail: &SchemaDiffDetail) -> SchemaDiffDetailFields {
    match detail {
        SchemaDiffDetail::AddedField {
            field,
            required,
            default_expression,
        } => SchemaDiffDetailFields::AddedField {
            field: field.clone(),
            required: *required,
            default_expression: default_expression.as_ref().map(|expr| expr.to_string()),
        },
        SchemaDiffDetail::RemovedField { field } => SchemaDiffDetailFields::RemovedField {
            field: field.clone(),
        },
        SchemaDiffDetail::TypeChanged {
            field,
            from_type,
            to_type,
        } => SchemaDiffDetailFields::TypeChanged {
            field: field.clone(),
            from_type: from_type.to_string(),
            to_type: to_type.to_string(),
        },
        SchemaDiffDetail::EnumDomainExpanded {
            field,
            added_variants,
        } => SchemaDiffDetailFields::EnumDomainExpanded {
            field: field.clone(),
            added_variants: added_variants
                .iter()
                .map(|variant| variant.to_string())
                .collect(),
        },
        SchemaDiffDetail::InvariantContractChanged { contract_name } => {
            SchemaDiffDetailFields::InvariantContractChanged {
                contract_name: contract_name.to_string(),
            }
        }
        SchemaDiffDetail::ProjectionContractChanged { projection_name } => {
            SchemaDiffDetailFields::ProjectionContractChanged {
                projection_name: projection_name.to_string(),
            }
        }
        SchemaDiffDetail::SubscriberContractChanged { contract_name } => {
            SchemaDiffDetailFields::SubscriberContractChanged {
                contract_name: contract_name.to_string(),
            }
        }
        SchemaDiffDetail::FreeText {
            detail,
            declared_intent,
        } => SchemaDiffDetailFields::FreeText {
            detail: detail.to_string(),
            declared_intent: *declared_intent,
        },
        &_ => unreachable!("unhandled Relational contract variant"),
    }
}
