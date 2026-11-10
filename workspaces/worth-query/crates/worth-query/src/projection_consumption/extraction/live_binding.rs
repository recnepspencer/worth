use super::super::consumed::{
    ConsumedEntityIdentityFact, ConsumedFieldValueFact, ConsumedProjectionContractProvenance,
    ConsumedProjectionFactInventory, ConsumedProjectionFactSet, ConsumedProjectionSourceTruth,
    ConsumedSourceReferenceFact, ConsumedViewLocalIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::MaterializedProjectionContract;
use super::super::facts::ProjectionFactKind;
use super::super::identity::compose_live_binding_row_identity;
use super::super::source::ProjectionSourceFamily;
use super::row_materialization::query_read_result_row_fields;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionFactExtractionError;
use crate::runtime::{WorthQueryLiveArtifactBinding, WorthQueryLiveArtifactTarget};

pub(super) fn extract_live_binding_facts(
    contract: &MaterializedProjectionContract,
    binding: &WorthQueryLiveArtifactBinding,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::ensure_contract_family(contract, ProjectionSourceFamily::LiveArtifactBinding)?;
    super::ensure_source_identity(contract.source_identity(), binding.binding_digest())?;
    let mut extracted = ExtractedLiveBindingFacts::for_contract(contract);
    for target in binding.targets() {
        let read = binding
            .read_for_target(target)
            .expect("binding targets should resolve");
        extracted.extract_target_rows(
            contract,
            binding.binding_digest(),
            target.view_name(),
            read,
        )?;
    }
    let source_references = if extracted.extracts_source_references {
        binding_target_source_references("live_target_view", binding.targets())
    } else {
        Vec::new()
    };
    if extracted.extracts_source_references
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
    Ok(extracted.into_fact_set(contract, source_references)?)
}

struct ExtractedLiveBindingFacts {
    extracts_entity_identity: bool,
    extracts_view_local_identity: bool,
    extracts_source_references: bool,
    requested_field_lookups: usize,
    row_count: usize,
    entity_identities: Vec<ConsumedEntityIdentityFact>,
    view_local_identities: Vec<ConsumedViewLocalIdentityFact>,
    display_fields: Vec<ConsumedFieldValueFact>,
    derived_fields: Vec<ConsumedFieldValueFact>,
}

impl ExtractedLiveBindingFacts {
    fn for_contract(contract: &MaterializedProjectionContract) -> Self {
        let has_kind = |kind| {
            contract
                .fact_families()
                .iter()
                .any(|fact| fact.kind() == kind)
        };
        Self {
            extracts_entity_identity: has_kind(ProjectionFactKind::EntityIdentity),
            extracts_view_local_identity: has_kind(ProjectionFactKind::ViewLocalIdentity),
            extracts_source_references: has_kind(ProjectionFactKind::SourceReference),
            requested_field_lookups: contract
                .fact_families()
                .iter()
                .filter(|fact| {
                    matches!(
                        fact.kind(),
                        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField
                    )
                })
                .count(),
            row_count: 0,
            entity_identities: Vec::new(),
            view_local_identities: Vec::new(),
            display_fields: Vec::new(),
            derived_fields: Vec::new(),
        }
    }

    fn extract_target_rows(
        &mut self,
        contract: &MaterializedProjectionContract,
        binding_digest: &str,
        view_name: &str,
        read: &crate::runtime::WorthQueryLiveReadResult,
    ) -> Result<(), ProjectionFactExtractionError> {
        for (index, row) in read.rows().iter().enumerate() {
            self.row_count += 1;
            let row_identity = live_binding_row_identity(binding_digest, view_name, index)?;
            let row_fields = query_read_result_row_fields(contract, row)?;
            for fact_family in contract.fact_families() {
                match fact_family.kind() {
                    ProjectionFactKind::EntityIdentity => {
                        self.entity_identities.push(ConsumedEntityIdentityFact::new(
                            row_identity.as_str(),
                            row.identity().clone(),
                        ))
                    }
                    ProjectionFactKind::ViewLocalIdentity => {
                        self.view_local_identities
                            .push(ConsumedViewLocalIdentityFact::new(
                                row_identity.as_str(),
                                row_identity.as_str(),
                            ))
                    }
                    ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
                        let field_path = fact_family
                            .field_path()
                            .expect("field path required")
                            .clone();
                        let value = super::row_materialization::native_value_or_absence(
                            contract,
                            fact_family,
                            row_identity.as_str(),
                            row_fields.get(&field_path),
                        )?;
                        let fact = ConsumedFieldValueFact::new_from_bound_family(
                            contract,
                            row_identity.as_str(),
                            fact_family,
                            value,
                        );
                        if fact_family.kind() == ProjectionFactKind::DisplayField {
                            self.display_fields.push(fact);
                        } else {
                            self.derived_fields.push(fact);
                        }
                    }
                    ProjectionFactKind::TargetIdentity
                    | ProjectionFactKind::SourceReference
                    | ProjectionFactKind::EffectContinuity
                    | ProjectionFactKind::Membership
                    | ProjectionFactKind::RelationEndpoint => {}
                }
            }
        }
        Ok(())
    }

    fn into_fact_set(
        self,
        contract: &MaterializedProjectionContract,
        source_references: Vec<ConsumedSourceReferenceFact>,
    ) -> Result<ConsumedProjectionFactSet, IdentityDerivationDenial> {
        let row_identity_surface_count =
            usize::from(self.extracts_entity_identity || self.extracts_view_local_identity);
        let extracted_fact_count = self.entity_identities.len()
            + self.view_local_identities.len()
            + self.display_fields.len()
            + self.derived_fields.len()
            + source_references.len();
        let counters = ProjectionFactExtractionCounters::new(
            contract.fact_families().len(),
            contract.fact_families().len(),
            extracted_fact_count,
            self.row_count * (self.requested_field_lookups + row_identity_surface_count),
            source_references.len(),
        );
        ConsumedProjectionFactSet::new(
            ConsumedProjectionContractProvenance::from_contract(contract),
            ConsumedProjectionSourceTruth::from_contract(
                contract,
                crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(
                    contract,
                    self.row_count,
                ),
            ),
            counters,
            ConsumedProjectionFactInventory {
                entity_identities: self.entity_identities,
                view_local_identities: self.view_local_identities,
                memberships: Vec::new(),
                display_fields: self.display_fields,
                derived_fields: self.derived_fields,
                target_identities: Vec::new(),
                source_references,
                effect_continuity_facts: Vec::new(),
                relation_endpoints: Vec::new(),
            },
        )
    }
}

fn live_binding_row_identity(
    binding_digest: &str,
    view_name: &str,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    compose_live_binding_row_identity(binding_digest, view_name, index)
}

fn binding_target_source_references<'a>(
    label: &'static str,
    targets: impl IntoIterator<Item = &'a WorthQueryLiveArtifactTarget>,
) -> Vec<ConsumedSourceReferenceFact> {
    targets
        .into_iter()
        .map(|target| ConsumedSourceReferenceFact::new(label, target.view_name()))
        .collect()
}
