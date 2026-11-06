use super::super::super::consumed::{
    ConsumedEntityIdentityFact, ConsumedFieldValueFact, ConsumedNativeValue,
    ConsumedProjectionContractProvenance, ConsumedProjectionFactInventory,
    ConsumedProjectionFactSet, ConsumedProjectionSourceTruth, ConsumedViewLocalIdentityFact,
    ProjectionFactExtractionCounters,
};
use super::super::super::contracts::{BoundProjectionFactFamily, MaterializedProjectionContract};
use super::super::super::facts::{ProjectionFactFieldPath, ProjectionFactKind};
use super::super::super::identity::compose_scoped_row_source_identity;
use super::super::ProjectionFactExtractionError;
use super::row_like_field_paths::identity_field_path;
use super::row_like_values::consumed_aspect_value_as_str;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;

#[derive(Clone, Copy)]
pub(super) enum RowIdentityExtractionMode {
    RowIdentityAsEntityIdentity,
    IdentityFieldBackedEntityIdentity,
}

pub(super) fn extract_materialized_rows<RowData, Lookup>(
    contract: &MaterializedProjectionContract,
    rows: &[(String, Option<WorthQueryEntityIdentity>, RowData)],
    lookup: Lookup,
    row_identity_mode: RowIdentityExtractionMode,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError>
where
    Lookup: for<'a> Fn(
        &'a str,
        &'a RowData,
        &'a ProjectionFactFieldPath,
        ProjectionFactKind,
    )
        -> Result<Option<&'a ConsumedNativeValue>, ProjectionFactExtractionError>,
{
    let shape = ExtractionShape::from_contract(contract, row_identity_mode);
    let mut facts = ExtractedRowFacts::default();
    for (row_identity, typed_identity, row_data) in rows {
        facts.extract_row(
            contract,
            row_identity,
            typed_identity,
            row_data,
            &lookup,
            row_identity_mode,
        )?;
    }
    Ok(facts.into_fact_set(contract, rows.len(), shape))
}

#[derive(Clone, Copy)]
struct ExtractionShape {
    row_width: usize,
}

impl ExtractionShape {
    fn from_contract(
        contract: &MaterializedProjectionContract,
        _mode: RowIdentityExtractionMode,
    ) -> Self {
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
        let entity = contract
            .fact_families()
            .iter()
            .any(|fact| fact.kind() == ProjectionFactKind::EntityIdentity);
        let view_local = contract
            .fact_families()
            .iter()
            .any(|fact| fact.kind() == ProjectionFactKind::ViewLocalIdentity);
        let identity_surfaces = usize::from(entity || view_local);
        Self {
            row_width: requested_field_lookups + identity_surfaces,
        }
    }
}

#[derive(Default)]
struct ExtractedRowFacts {
    entity_identities: Vec<ConsumedEntityIdentityFact>,
    view_local_identities: Vec<ConsumedViewLocalIdentityFact>,
    display_fields: Vec<ConsumedFieldValueFact>,
    derived_fields: Vec<ConsumedFieldValueFact>,
}

impl ExtractedRowFacts {
    fn extract_row<RowData, Lookup>(
        &mut self,
        contract: &MaterializedProjectionContract,
        row_identity: &str,
        typed_identity: &Option<WorthQueryEntityIdentity>,
        row_data: &RowData,
        lookup: &Lookup,
        mode: RowIdentityExtractionMode,
    ) -> Result<(), ProjectionFactExtractionError>
    where
        Lookup:
            for<'a> Fn(
                &'a str,
                &'a RowData,
                &'a ProjectionFactFieldPath,
                ProjectionFactKind,
            )
                -> Result<Option<&'a ConsumedNativeValue>, ProjectionFactExtractionError>,
    {
        let context = RowExtractionContext {
            contract,
            row_identity,
            typed_identity,
            row_data,
            lookup,
            mode,
        };
        for family in contract.fact_families() {
            self.extract_family(family, &context)?;
        }
        Ok(())
    }

    fn extract_family<RowData, Lookup>(
        &mut self,
        family: &BoundProjectionFactFamily,
        context: &RowExtractionContext<'_, RowData, Lookup>,
    ) -> Result<(), ProjectionFactExtractionError>
    where
        Lookup:
            for<'a> Fn(
                &'a str,
                &'a RowData,
                &'a ProjectionFactFieldPath,
                ProjectionFactKind,
            )
                -> Result<Option<&'a ConsumedNativeValue>, ProjectionFactExtractionError>,
    {
        match family.kind() {
            ProjectionFactKind::EntityIdentity => {
                self.entity_identities.push(ConsumedEntityIdentityFact::new(
                    context.row_identity,
                    extract_entity_identity(
                        context.contract,
                        context.row_identity,
                        context.typed_identity,
                        context.row_data,
                        context.lookup,
                        context.mode,
                    )?,
                ))
            }
            ProjectionFactKind::ViewLocalIdentity => {
                self.view_local_identities
                    .push(ConsumedViewLocalIdentityFact::new(
                        context.row_identity,
                        context.row_identity,
                    ))
            }
            ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
                let field_path = family.field_path().expect("field family carries a path");
                let observed = (context.lookup)(
                    context.row_identity,
                    context.row_data,
                    field_path,
                    family.kind(),
                )?;
                let value = super::native_field_resolution::native_value_or_absence(
                    context.contract,
                    family,
                    context.row_identity,
                    observed,
                )?;
                let fact = ConsumedFieldValueFact::new_from_bound_family(
                    context.contract,
                    context.row_identity,
                    family,
                    value,
                );
                if family.kind() == ProjectionFactKind::DisplayField {
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
        Ok(())
    }

    fn into_fact_set(
        self,
        contract: &MaterializedProjectionContract,
        row_count: usize,
        shape: ExtractionShape,
    ) -> Result<ConsumedProjectionFactSet, IdentityDerivationDenial> {
        let extracted_fact_count = self.entity_identities.len()
            + self.view_local_identities.len()
            + self.display_fields.len()
            + self.derived_fields.len();
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
                row_count * shape.row_width,
                0,
            ),
            ConsumedProjectionFactInventory {
                entity_identities: self.entity_identities,
                view_local_identities: self.view_local_identities,
                memberships: Vec::new(),
                display_fields: self.display_fields,
                derived_fields: self.derived_fields,
                target_identities: Vec::new(),
                source_references: Vec::new(),
                effect_continuity_facts: Vec::new(),
                relation_endpoints: Vec::new(),
            },
        )?)
    }
}

struct RowExtractionContext<'a, RowData, Lookup> {
    contract: &'a MaterializedProjectionContract,
    row_identity: &'a str,
    typed_identity: &'a Option<WorthQueryEntityIdentity>,
    row_data: &'a RowData,
    lookup: &'a Lookup,
    mode: RowIdentityExtractionMode,
}

fn extract_entity_identity<RowData, Lookup>(
    contract: &MaterializedProjectionContract,
    row_identity: &str,
    typed_identity: &Option<WorthQueryEntityIdentity>,
    row_data: &RowData,
    lookup: &Lookup,
    mode: RowIdentityExtractionMode,
) -> Result<WorthQueryEntityIdentity, ProjectionFactExtractionError>
where
    Lookup: for<'a> Fn(
        &'a str,
        &'a RowData,
        &'a ProjectionFactFieldPath,
        ProjectionFactKind,
    )
        -> Result<Option<&'a ConsumedNativeValue>, ProjectionFactExtractionError>,
{
    if matches!(mode, RowIdentityExtractionMode::RowIdentityAsEntityIdentity) {
        return Ok(typed_identity.clone().unwrap_or_else(|| {
            crate::memory_workspace::admit_authored_entity_label(row_identity)
        }));
    }
    let path = identity_field_path();
    let value = lookup(
        row_identity,
        row_data,
        &path,
        ProjectionFactKind::EntityIdentity,
    )?
    .ok_or_else(|| {
        super::native_field_resolution::missing_declared_field(
            contract,
            row_identity,
            &path,
            ProjectionFactKind::EntityIdentity,
        )
    })?;
    let label = value
        .view()
        .scalar()
        .and_then(consumed_aspect_value_as_str)
        .ok_or_else(
            || ProjectionFactExtractionError::InvalidDeclaredFieldValueShape {
                source_family: contract.source_family(),
                source_identity: compose_scoped_row_source_identity(
                    contract.source_identity(),
                    row_identity,
                ),
                field_key: "identity.id".to_string(),
                fact_kind: ProjectionFactKind::EntityIdentity,
                expected_shape: "string",
            },
        )?;
    Ok(crate::memory_workspace::admit_authored_entity_label(label))
}
