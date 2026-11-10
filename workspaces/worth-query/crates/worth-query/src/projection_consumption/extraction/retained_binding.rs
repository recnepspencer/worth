use super::super::consumed::{
    ConsumedFieldValueFact, ConsumedNativeValue, ConsumedProjectionContractProvenance,
    ConsumedProjectionFactInventory, ConsumedProjectionFactSet, ConsumedProjectionSourceTruth,
    ConsumedSourceReferenceFact, ConsumedViewLocalIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::MaterializedProjectionContract;
use super::super::facts::ProjectionFactKind;
use super::super::identity::compose_retained_binding_row_identity;
use super::super::source::ProjectionSourceFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionFactExtractionError;
use crate::runtime::{WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationTarget};
use crate::runtime::{WorthQueryRetainedFieldPath, WorthQueryRetainedValueView};

pub(super) fn extract_retained_binding_facts(
    contract: &MaterializedProjectionContract,
    binding: &WorthQueryDerivedArtifactBinding,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::ensure_contract_family(
        contract,
        ProjectionSourceFamily::RetainedDerivedArtifactBinding,
    )?;
    super::ensure_source_identity(contract.source_identity(), binding.binding_for_reporting())?;

    let extracts_view_local_identity = contract
        .fact_families()
        .iter()
        .any(|fact| fact.kind() == ProjectionFactKind::ViewLocalIdentity);
    let requested_field_lookups = contract
        .fact_families()
        .iter()
        .filter(|fact| {
            matches!(
                fact.kind(),
                ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField
            )
        })
        .count();
    let extracts_source_references = contract
        .fact_families()
        .iter()
        .any(|fact| fact.kind() == ProjectionFactKind::SourceReference);

    let mut view_local_identities = Vec::new();
    let mut display_fields = Vec::new();
    let mut derived_fields = Vec::new();
    let mut row_count = 0;

    for target in binding.targets() {
        let materialization = binding
            .materialization_for_target(target)
            .expect("binding targets should resolve");
        let view_name = target.view_name();
        for index in 0..materialization.retained_row_count() {
            row_count += 1;
            let row_identity =
                retained_binding_row_identity(binding.binding_for_reporting(), view_name, index)?;
            for fact_family in contract.fact_families() {
                match fact_family.kind() {
                    ProjectionFactKind::ViewLocalIdentity => {
                        view_local_identities.push(ConsumedViewLocalIdentityFact::new(
                            row_identity.as_str(),
                            row_identity.as_str(),
                        ));
                    }
                    ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
                        let consumed_field_path = fact_family
                            .field_path()
                            .expect("field path required")
                            .clone();
                        let field_path = retained_field_path(&consumed_field_path);
                        let observed = materialization
                            .retained_native_value_at_path(index, &field_path)
                            .map(|value| match value {
                                WorthQueryRetainedValueView::Scalar(value) => {
                                    ConsumedNativeValue::scalar(value.clone())
                                }
                                WorthQueryRetainedValueView::Struct(value) => {
                                    ConsumedNativeValue::struct_value(value.clone())
                                }
                            });
                        let value = super::row_materialization::native_value_or_absence(
                            contract,
                            fact_family,
                            row_identity.as_str(),
                            observed.as_ref(),
                        )?;
                        let fact = ConsumedFieldValueFact::new_from_bound_family(
                            contract,
                            row_identity.as_str(),
                            fact_family,
                            value,
                        );
                        if fact_family.kind() == ProjectionFactKind::DisplayField {
                            display_fields.push(fact);
                        } else {
                            derived_fields.push(fact);
                        }
                    }
                    ProjectionFactKind::EntityIdentity
                    | ProjectionFactKind::TargetIdentity
                    | ProjectionFactKind::SourceReference
                    | ProjectionFactKind::EffectContinuity
                    | ProjectionFactKind::Membership
                    | ProjectionFactKind::RelationEndpoint => {}
                }
            }
        }
    }

    let source_references = if extracts_source_references {
        binding_target_source_references("retained_target_view", binding.targets())
    } else {
        Vec::new()
    };
    if extracts_source_references
        && !super::source_reference_inventory_matches(
            contract.source_reference_identities(),
            &source_references,
        )
    {
        return Err(
            ProjectionFactExtractionError::SourceReferenceEvidenceMismatch {
                expected_count: contract.source_reference_identities().len(),
                actual_count: source_references.len(),
            },
        );
    }

    let row_width_per_row = requested_field_lookups + usize::from(extracts_view_local_identity);
    let extracted_fact_count = view_local_identities.len()
        + display_fields.len()
        + derived_fields.len()
        + source_references.len();

    Ok(ConsumedProjectionFactSet::new(
        ConsumedProjectionContractProvenance::from_contract(contract),
        ConsumedProjectionSourceTruth::from_contract(
            contract,
            crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(
                contract, row_count,
            ),
        ),
        ProjectionFactExtractionCounters::new(
            contract.fact_families().len(),
            contract.fact_families().len(),
            extracted_fact_count,
            row_count * row_width_per_row,
            source_references.len(),
        ),
        ConsumedProjectionFactInventory {
            entity_identities: Vec::new(),
            view_local_identities,
            memberships: Vec::new(),
            display_fields,
            derived_fields,
            target_identities: Vec::new(),
            source_references,
            effect_continuity_facts: Vec::new(),
            relation_endpoints: Vec::new(),
        },
    )?)
}

fn retained_field_path(
    path: &crate::projection_consumption::ProjectionFactFieldPath,
) -> WorthQueryRetainedFieldPath {
    if let Some(aspect) = path.native_aspect_key() {
        return match path.native_field_key() {
            Some(field) => {
                WorthQueryRetainedFieldPath::from_native_keys(aspect.clone(), field.clone())
            }
            None => WorthQueryRetainedFieldPath::from_native_aspect_key(aspect.clone()),
        };
    }
    WorthQueryRetainedFieldPath::from_canonical_field_path(
        path.canonical_field_path()
            .expect("non-native projection path remains canonical")
            .clone(),
    )
}

fn retained_binding_row_identity(
    binding_digest: &str,
    view_name: &str,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    compose_retained_binding_row_identity(binding_digest, view_name, index)
}

fn binding_target_source_references<'a>(
    label: &'static str,
    targets: impl IntoIterator<Item = &'a WorthQueryDerivedMaterializationTarget>,
) -> Vec<ConsumedSourceReferenceFact> {
    targets
        .into_iter()
        .map(|target| ConsumedSourceReferenceFact::new(label, target.view_name()))
        .collect()
}
