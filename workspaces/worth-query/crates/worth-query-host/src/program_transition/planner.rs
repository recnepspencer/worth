//! Compile a declarative [`WorthQueryProgramTransitionDescriptor`] onto a real
//! [`WorthQueryHomeProgramTransitionPlan`] using bounded source scans and
//! deterministic `BTreeMap` grouping.

use crate::application_home::{
    WorthQueryHomeProgramTransitionPlan, WorthQueryHomeProgramTransitionStep,
};
use worth_foundational::facade::FieldKey;
use worth_query_declaration::facade::program_transition::WorthQueryProgramTransitionDescriptor;
use worth_query_execution::facade::{
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};
use worth_relational::facade::{
    schema::{
        FreeFormSchemaDiffIntent, HistoricalInterpretationSensitivity, ProposedSchemaTransition,
        SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId,
        SchemaPublicationImpact, SchemaReconciliationPolicy, SchemaStratum, SchemaSubscriberImpact,
        SchemaVersionId,
    },
    transactions::WorkerIntentBatch,
};

use super::backfill::build_default_backfill_batches;

pub(super) struct CompiledTransitionPlan {
    pub(super) plan: WorthQueryHomeProgramTransitionPlan,
    pub(super) backfilled_records: usize,
}

pub(super) struct PlanInput<'a> {
    pub(super) descriptor: &'a WorthQueryProgramTransitionDescriptor,
    pub(super) max_entities_per_kind: usize,
}

pub(super) fn compile_declarative_plan<CustomStep>(
    input: PlanInput<'_>,
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
    custom_step: CustomStep,
) -> Result<CompiledTransitionPlan, WorthQueryPrimaryGraphInstallationDenial>
where
    CustomStep: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    )
        -> Result<Vec<WorkerIntentBatch>, WorthQueryPrimaryGraphInstallationDenial>,
{
    verify_declared_members(input.descriptor, source, target)?;
    let schema_transition = build_schema_transition(input.descriptor, source, target)?;
    let (mut batches, backfilled_records) = build_default_backfill_batches(
        input.descriptor,
        input.max_entities_per_kind,
        source,
        target,
    )?;
    batches.extend(custom_step(source, target)?);

    Ok(CompiledTransitionPlan {
        plan: WorthQueryHomeProgramTransitionPlan {
            steps: vec![WorthQueryHomeProgramTransitionStep { batches }],
            target: 0,
            schema_transition,
        },
        backfilled_records,
    })
}

fn verify_declared_members(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for retained in descriptor.retained_records() {
        let entity = retained.entity_name();
        if source.entity_kind(entity).is_none() || target.entity_kind(entity).is_none() {
            return Err(schema_denial(format!(
                "retained entity `{entity}` must exist in both predecessor and target schemas"
            )));
        }
    }
    for added in descriptor.added_records() {
        let entity = added.entity_name();
        if target.entity_kind(entity).is_none() {
            return Err(schema_denial(format!(
                "added entity `{entity}` is absent from target schema"
            )));
        }
    }
    for index in descriptor.index_rebuilds() {
        if target
            .planned_field(index.entity(), index.aspect(), index.field())
            .is_none()
        {
            return Err(schema_denial(format!(
                "index rebuild target `{}.{}.{}` is absent from target schema",
                index.entity(),
                index.aspect(),
                index.field()
            )));
        }
    }
    Ok(())
}

fn build_schema_transition(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<
    Option<(ProposedSchemaTransition, Option<SchemaReconciliationPolicy>)>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let (before_id, before_version) = read_schema_basis(source.schema_registry(), "predecessor")?;
    let (after_id, after_version) = read_schema_basis(target.schema_registry(), "target")?;

    let mut diff_atoms = added_record_atoms(descriptor, target, &after_id, after_version);
    diff_atoms.extend(added_default_atoms(
        descriptor,
        target,
        &after_id,
        after_version,
    )?);

    if diff_atoms.is_empty() {
        return Ok(None);
    }
    Ok(Some((
        ProposedSchemaTransition {
            source_schema_id: before_id,
            source_schema_version_id: before_version,
            target_schema_id: after_id,
            target_schema_version_id: after_version,
            diff_atoms,
        },
        None,
    )))
}

fn read_schema_basis(
    registry: &worth_relational::facade::schema::RelationalSchemaRegistry,
    role: &str,
) -> Result<(SchemaId, SchemaVersionId), WorthQueryPrimaryGraphInstallationDenial> {
    registry
        .authoritative_schema_basis()
        .map_err(|cause| schema_denial(format!("{role} schema basis refused: {cause:?}")))?
        .ok_or_else(|| schema_denial(format!("{role} schema basis is absent")))
}

fn added_record_atoms(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
    after_id: &SchemaId,
    after_version: SchemaVersionId,
) -> Vec<SchemaDiffAtom> {
    descriptor
        .added_records()
        .iter()
        .map(|added| {
            let entity = added.entity_name();
            SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::EntityKind,
                    after_id.clone(),
                    after_version,
                    target.entity_kind(entity),
                    entity,
                ),
                vec![
                    SchemaStratum::StructuralShape,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::None,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::FreeText {
                    detail: descriptor.description().into(),
                    declared_intent: FreeFormSchemaDiffIntent::Additive,
                },
            )
        })
        .collect()
}

fn added_default_atoms(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
    after_id: &SchemaId,
    after_version: SchemaVersionId,
) -> Result<Vec<SchemaDiffAtom>, WorthQueryPrimaryGraphInstallationDenial> {
    let mut atoms = Vec::with_capacity(descriptor.added_defaults().len());
    for added_default in descriptor.added_defaults() {
        let field_key = FieldKey::new(added_default.field()).ok_or_else(|| {
            schema_denial(format!(
                "invalid default field key `{}`",
                added_default.field()
            ))
        })?;
        atoms.push(SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                after_id.clone(),
                after_version,
                target.entity_kind(added_default.entity()),
                added_default.field(),
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key,
                required: added_default.required(),
                default_expression: Some(added_default.default_label().into()),
            },
        ));
    }
    Ok(atoms)
}

fn schema_denial(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        detail,
    )
}
