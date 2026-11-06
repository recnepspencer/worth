use super::super::consumed::{
    ConsumedEntityIdentityFact, ConsumedFieldValueFact, ConsumedProjectionContractProvenance,
    ConsumedProjectionFactInventory, ConsumedProjectionFactSet, ConsumedProjectionSourceTruth,
    ConsumedSourceReferenceFact, ConsumedViewLocalIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::MaterializedProjectionContract;
use super::super::facts::ProjectionFactKind;
use super::super::identity::compose_query_context_row_identity;
use super::super::source::ProjectionSourceFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionFactExtractionError;
use crate::query_context::QueryContextExecutionArtifact;

pub(super) fn extract_query_context_facts(
    contract: &MaterializedProjectionContract,
    execution: &QueryContextExecutionArtifact,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    admit_query_context_source(contract, execution)?;
    let requested_facts = RequestedQueryContextFacts::from_contract(contract);
    let extracted_rows = extract_query_context_rows(contract, execution);
    let source_references = extract_query_context_source_references(
        contract,
        execution,
        requested_facts.source_references,
    )?;

    Ok(build_query_context_fact_set(
        QueryContextFactSetBuildInput {
            contract,
            execution,
            requested: requested_facts,
            extracted: extracted_rows,
            source_references,
        },
    ))
}

#[derive(Clone, Copy)]
struct RequestedQueryContextFacts {
    entity_identity: bool,
    view_local_identity: bool,
    row_value_fields: bool,
    source_references: bool,
}

impl RequestedQueryContextFacts {
    fn from_contract(contract: &MaterializedProjectionContract) -> Self {
        Self {
            entity_identity: contract
                .fact_families()
                .iter()
                .any(|fact| fact.kind() == ProjectionFactKind::EntityIdentity),
            view_local_identity: contract
                .fact_families()
                .iter()
                .any(|fact| fact.kind() == ProjectionFactKind::ViewLocalIdentity),
            row_value_fields: contract.fact_families().iter().any(|fact| {
                matches!(
                    fact.kind(),
                    ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField
                )
            }),
            source_references: contract
                .fact_families()
                .iter()
                .any(|fact| fact.kind() == ProjectionFactKind::SourceReference),
        }
    }
}

#[derive(Default)]
struct ExtractedQueryContextRows {
    entity_identities: Vec<ConsumedEntityIdentityFact>,
    view_local_identities: Vec<ConsumedViewLocalIdentityFact>,
    display_fields: Vec<ConsumedFieldValueFact>,
    derived_fields: Vec<ConsumedFieldValueFact>,
}

fn admit_query_context_source(
    contract: &MaterializedProjectionContract,
    execution: &QueryContextExecutionArtifact,
) -> Result<(), ProjectionFactExtractionError> {
    super::ensure_contract_family(contract, ProjectionSourceFamily::QueryContextExecution)?;
    super::ensure_source_identity(
        contract.source_identity(),
        &query_context_source_identity(execution),
    )
}

fn extract_query_context_rows(
    contract: &MaterializedProjectionContract,
    execution: &QueryContextExecutionArtifact,
) -> Result<ExtractedQueryContextRows, IdentityDerivationDenial> {
    let mut extracted = ExtractedQueryContextRows::default();
    for (index, row) in execution.rows().iter().enumerate() {
        let row_identity = query_context_row_identity(execution, index)?;
        let row_value =
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(row.clone());
        for fact_family in contract.fact_families() {
            match fact_family.kind() {
                ProjectionFactKind::EntityIdentity => {
                    extracted
                        .entity_identities
                        .push(ConsumedEntityIdentityFact::new(
                            row_identity.clone(),
                            crate::memory_workspace::admit_authored_entity_label(
                                row_identity.clone(),
                            )?,
                        ));
                }
                ProjectionFactKind::ViewLocalIdentity => {
                    extracted
                        .view_local_identities
                        .push(ConsumedViewLocalIdentityFact::new(
                            row_identity.clone(),
                            row_identity.clone(),
                        ));
                }
                ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
                    let field_path = fact_family
                        .field_path()
                        .expect("field path required")
                        .clone();
                    let fact = ConsumedFieldValueFact::new(
                        contract,
                        row_identity.clone(),
                        field_path,
                        row_value.clone(),
                    );
                    if fact_family.kind() == ProjectionFactKind::DisplayField {
                        extracted.display_fields.push(fact);
                    } else {
                        extracted.derived_fields.push(fact);
                    }
                }
                ProjectionFactKind::TargetIdentity
                | ProjectionFactKind::EffectContinuity
                | ProjectionFactKind::Membership
                | ProjectionFactKind::RelationEndpoint
                | ProjectionFactKind::SourceReference => {}
            }
        }
    }
    Ok(extracted)
}

fn extract_query_context_source_references(
    contract: &MaterializedProjectionContract,
    execution: &QueryContextExecutionArtifact,
    requested: bool,
) -> Result<Vec<ConsumedSourceReferenceFact>, ProjectionFactExtractionError> {
    let source_references = if requested {
        query_context_source_references(execution)
    } else {
        Vec::new()
    };
    if requested && !source_reference_inventory_matches(contract, &source_references) {
        return Err(
            ProjectionFactExtractionError::SourceReferenceEvidenceMismatch {
                expected_count: contract.source_reference_identities().len(),
                actual_count: source_references.len(),
            },
        );
    }
    Ok(source_references)
}

struct QueryContextFactSetBuildInput<'a> {
    contract: &'a MaterializedProjectionContract,
    execution: &'a QueryContextExecutionArtifact,
    requested: RequestedQueryContextFacts,
    extracted: ExtractedQueryContextRows,
    source_references: Vec<ConsumedSourceReferenceFact>,
}

fn build_query_context_fact_set(
    input: QueryContextFactSetBuildInput<'_>,
) -> Result<ConsumedProjectionFactSet, IdentityDerivationDenial> {
    let QueryContextFactSetBuildInput {
        contract,
        execution,
        requested,
        extracted,
        source_references,
    } = input;
    let row_identity_surface_count =
        usize::from(requested.entity_identity || requested.view_local_identity);
    let row_value_surface_count = usize::from(requested.row_value_fields);
    let source_row_width_consumed =
        execution.rows().len() * (row_identity_surface_count + row_value_surface_count);
    let source_evidence_lookup_width = source_references.len();
    let extracted_fact_count = extracted.entity_identities.len()
        + extracted.view_local_identities.len()
        + extracted.display_fields.len()
        + extracted.derived_fields.len()
        + source_references.len();

    Ok(ConsumedProjectionFactSet::new(
        ConsumedProjectionContractProvenance::from_contract(contract),
        ConsumedProjectionSourceTruth::from_contract(
            contract,
            crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(
                contract,
                execution.rows().len(),
            ),
        ),
        ProjectionFactExtractionCounters::new(
            contract.fact_families().len(),
            contract.fact_families().len(),
            extracted_fact_count,
            source_row_width_consumed,
            source_evidence_lookup_width,
        ),
        ConsumedProjectionFactInventory {
            entity_identities: extracted.entity_identities,
            view_local_identities: extracted.view_local_identities,
            memberships: Vec::new(),
            display_fields: extracted.display_fields,
            derived_fields: extracted.derived_fields,
            target_identities: Vec::new(),
            source_references,
            effect_continuity_facts: Vec::new(),
            relation_endpoints: Vec::new(),
        },
    )?)
}

fn query_context_row_identity(
    execution: &QueryContextExecutionArtifact,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_query_context_row_identity(
        execution.family().as_str(),
        index,
    )?)
}

fn query_context_source_identity(execution: &QueryContextExecutionArtifact) -> String {
    execution
        .materialization_path_identity()
        .unwrap_or_else(|| execution.family().as_str())
        .to_string()
}

fn query_context_source_references(
    execution: &QueryContextExecutionArtifact,
) -> Vec<ConsumedSourceReferenceFact> {
    let mut references = Vec::new();
    if let Some(materialization_path_identity) = execution.materialization_path_identity() {
        references.push(ConsumedSourceReferenceFact::new(
            "query_context_materialization_path",
            materialization_path_identity,
        ));
    }
    if let Some(preview_provenance_identity) = execution.preview_provenance_identity() {
        references.push(ConsumedSourceReferenceFact::new(
            "query_context_preview_provenance",
            preview_provenance_identity,
        ));
    }
    references
}

fn source_reference_inventory_matches(
    contract: &MaterializedProjectionContract,
    actual: &[ConsumedSourceReferenceFact],
) -> bool {
    contract.source_reference_identities().len() == actual.len()
        && contract
            .source_reference_identities()
            .iter()
            .zip(actual.iter())
            .all(|(expected, actual)| {
                expected.label() == actual.label() && expected.identity() == actual.identity()
            })
}
