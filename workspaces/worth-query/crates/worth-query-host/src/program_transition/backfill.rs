//! Lower declarative field defaults into atomic native entity backfill intents
//! under full cooperative work metering and retained-memory admission.
//!
//! When an entity already carries the target aspect on disk (e.g., adding a new
//! field to an existing aspect), this emits [`UpdateEntityFieldsIntent`]. When
//! the target aspect is newly introduced on the successor schema (e.g., an
//! `Extend<Task, (Option<TaskPriority>,), ()>` aspect absent from predecessor
//! records), this attaches the aspect atomically via
//! [`ApplyEntityAspectPatchIntent`] + [`PortableAspectPatchOperation::SetWhole`].

use std::collections::BTreeMap;

use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use worth_foundational::facade::{
    AspectContract, AspectFieldLocator, AspectKey, AspectShape, AspectValue,
    AuthoritativeRecordAspectState, ContractValidationInput, FieldKey, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};
use worth_query_declaration::facade::program_transition::{
    WorthQueryProgramTransitionDescriptor, WorthQueryProgramTransitionFieldDefault,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial,
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
        WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
    },
};
use worth_relational::facade::{
    identity::EntityId,
    transactions::{
        ApplyEntityAspectPatchIntent, AspectFieldPatch, EntityMutationIntent, MutationIntent,
        UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

const BYTES_PER_FIELD_INTENT_ESTIMATE: u64 = 128;

pub(super) struct BackfillPlan {
    pub(super) batches: Vec<WorkerIntentBatch>,
    pub(super) backfilled_records: usize,
    pub(super) examined_slots: usize,
}

struct AspectDefaultGroup {
    aspect_key: AspectKey,
    contract: AspectContract,
    locators: BTreeMap<AspectFieldLocator, AspectValue>,
}

struct BackfillRequest<'a, 'b, 'c> {
    descriptor: &'a WorthQueryProgramTransitionDescriptor,
    max_entities_per_kind: usize,
    source: &'a WorthQueryHomeProgramTransitionReadSource<'b, 'c>,
    target: &'a WorthQueryHomeProgramTransitionTarget<'a>,
}

pub(super) fn build_default_backfill_batches(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    max_entities_per_kind: usize,
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<BackfillPlan, WorthQueryPrimaryGraphInstallationDenial> {
    let request = source.execution_request();
    let backfill_req = BackfillRequest {
        descriptor,
        max_entities_per_kind,
        source,
        target,
    };
    let mut inner: Result<BackfillPlan, WorthQueryPrimaryGraphInstallationDenial> =
        Ok(BackfillPlan {
            batches: Vec::new(),
            backfilled_records: 0,
            examined_slots: 0,
        });
    let stage = request.run_native_with_checkpoint(|context| {
        inner = build_metered_backfill(&backfill_req, request, context);
    });
    match (stage, inner) {
        (Ok(_), result) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Err(denial)) => Err(denial),
        (Err(cause), _) => Err(work_ceiling_denial(cause)),
    }
}

fn build_metered_backfill(
    input: &BackfillRequest<'_, '_, '_>,
    request: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<BackfillPlan, WorthQueryPrimaryGraphInstallationDenial> {
    let mut held_memory: Vec<ExecutionMemoryReservation> = Vec::new();
    let mut by_entity: BTreeMap<&'static str, Vec<&WorthQueryProgramTransitionFieldDefault>> =
        BTreeMap::new();
    for field_default in input.descriptor.added_defaults() {
        charge_work(context, 1)?;
        by_entity
            .entry(field_default.entity())
            .or_default()
            .push(field_default);
    }

    let mut batches = Vec::with_capacity(by_entity.len());
    let mut backfilled_records = 0usize;
    let mut examined_slots = 0usize;

    for (entity_name, defaults) in by_entity {
        charge_work(context, 1)?;
        let groups = resolve_aspect_groups(entity_name, &defaults, input.target, context)?;
        let rows = input
            .source
            .entities(entity_name, input.max_entities_per_kind)?;
        examined_slots = examined_slots.saturating_add(rows.entity_slots_examined());
        if rows.records().is_empty() {
            continue;
        }
        let mut batch = WorkerIntentBatch::new(format!(
            "transition-default-backfill:{}:{entity_name}",
            input.descriptor.transition_name()
        ));
        for row in rows.records() {
            let field_count = defaults.len().max(1) as u64;
            charge_work(context, field_count)?;
            let estimated_bytes = field_count.saturating_mul(BYTES_PER_FIELD_INTENT_ESTIMATE);
            if let Some(reservation) = request
                .reserve_retained_memory_in_stage(context, estimated_bytes)
                .map_err(work_ceiling_denial)?
            {
                held_memory.push(reservation);
            }
            batch = push_entity_backfill_intents(
                batch,
                row.entity_id,
                row.authoritative_aspect_state.as_ref(),
                &groups,
            )?;
            backfilled_records = backfilled_records.saturating_add(1);
        }
        batches.push(batch);
    }

    drop(held_memory);
    Ok(BackfillPlan {
        batches,
        backfilled_records,
        examined_slots,
    })
}

fn resolve_aspect_groups(
    entity_name: &'static str,
    defaults: &[&WorthQueryProgramTransitionFieldDefault],
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<AspectDefaultGroup>, WorthQueryPrimaryGraphInstallationDenial> {
    let kind_id = target
        .entity_kind(entity_name)
        .ok_or_else(|| schema_denial(format!("entity `{entity_name}` is absent from target")))?;
    let registration = target
        .schema_registry()
        .entity_registration(kind_id)
        .map_err(|cause| {
            schema_denial(format!(
                "target entity `{entity_name}` registration missing: {cause:?}"
            ))
        })?;

    let mut grouped: BTreeMap<AspectKey, BTreeMap<AspectFieldLocator, AspectValue>> =
        BTreeMap::new();
    for field_default in defaults {
        charge_work(context, 1)?;
        let locator = target
            .planned_field(entity_name, field_default.aspect(), field_default.field())
            .ok_or_else(|| {
                schema_denial(format!(
                    "default field `{}.{}.{}` is absent from target schema",
                    entity_name,
                    field_default.aspect(),
                    field_default.field()
                ))
            })?;
        grouped
            .entry(locator.aspect().aspect_key().clone())
            .or_default()
            .insert(locator, field_default.default_value().clone());
    }

    let mut groups = Vec::with_capacity(grouped.len());
    for (aspect_key, locators) in grouped {
        charge_work(context, 1)?;
        let binding = registration
            .aspect_contract_declarations
            .aspects
            .iter()
            .find(|declared| declared.aspect_key() == aspect_key)
            .ok_or_else(|| {
                schema_denial(format!(
                    "target aspect `{aspect_key:?}` on `{entity_name}` has no declared contract"
                ))
            })?;
        groups.push(AspectDefaultGroup {
            aspect_key,
            contract: binding.contract.clone(),
            locators,
        });
    }
    Ok(groups)
}

fn push_entity_backfill_intents(
    mut batch: WorkerIntentBatch,
    entity_id: EntityId,
    aspect_state: Option<&AuthoritativeRecordAspectState>,
    groups: &[AspectDefaultGroup],
) -> Result<WorkerIntentBatch, WorthQueryPrimaryGraphInstallationDenial> {
    let mut existing_aspect_fields: BTreeMap<AspectFieldLocator, AspectValue> = BTreeMap::new();
    let mut attach_ops: Vec<PortableAspectPatchOperation> = Vec::new();

    for group in groups {
        let already_attached =
            aspect_state.is_some_and(|state| state.get(&group.aspect_key).is_some());
        if already_attached {
            for (locator, value) in &group.locators {
                existing_aspect_fields.insert(locator.clone(), value.clone());
            }
        } else {
            attach_ops.push(build_set_whole_operation(group)?);
        }
    }

    if !attach_ops.is_empty() {
        batch = batch.push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id,
                aspect_patch: PortableRecordAspectPatch::new(attach_ops),
            }),
        ));
    }
    if !existing_aspect_fields.is_empty() {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id,
                fields: AspectFieldPatch::new(existing_aspect_fields),
            },
        )));
    }
    Ok(batch)
}

fn build_set_whole_operation(
    group: &AspectDefaultGroup,
) -> Result<PortableAspectPatchOperation, WorthQueryPrimaryGraphInstallationDenial> {
    let basis = PortableAspectContractBasis::from_contract(&group.contract);
    let value = match group.contract.shape() {
        AspectShape::Struct(_) => {
            let mut entries: Vec<(FieldKey, AspectValue)> =
                Vec::with_capacity(group.locators.len());
            for (locator, field_value) in &group.locators {
                let [field_key] = locator.field_path().fields() else {
                    return Err(schema_denial(format!(
                        "nested struct default path `{:?}` is not supported for aspect attachment",
                        locator.field_path()
                    )));
                };
                entries.push((field_key.clone(), field_value.clone()));
            }
            let struct_value = StructAspectValue::new(entries).map_err(|cause| {
                schema_denial(format!(
                    "failed to construct struct aspect default for `{:?}`: {cause:?}",
                    group.aspect_key
                ))
            })?;
            ContractValidationInput::Struct(struct_value)
        }
        AspectShape::Scalar(_) | AspectShape::Opaque(_) | AspectShape::Reference(_) => {
            if group.locators.len() != 1 {
                return Err(schema_denial(format!(
                    "scalar aspect `{:?}` received {} defaults",
                    group.aspect_key,
                    group.locators.len()
                )));
            }
            let (_, scalar) = group.locators.iter().next().expect("checked len == 1");
            ContractValidationInput::Scalar(scalar.clone())
        }
    };
    Ok(PortableAspectPatchOperation::SetWhole { basis, value })
}

fn charge_work(
    context: &mut MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    context
        .checkpoint(units)
        .map_err(|stop| work_ceiling_denial(WorkCeilingDenial::Stopped(stop)))
}

fn work_ceiling_denial(cause: WorkCeilingDenial) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match cause {
        WorkCeilingDenial::Stopped(MapKernelStop::Cancelled) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                kind: WorthQueryProviderSessionControlStopKind::Cancelled,
            }
        }
        WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                kind: WorthQueryProviderSessionControlStopKind::TimedOut,
            }
        }
        WorkCeilingDenial::Stopped(_) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            }
        }
        WorkCeilingDenial::Admission(_) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            }
        }
        WorkCeilingDenial::Panicked => {
            WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
        }
    };
    WorthQueryPrimaryGraphInstallationDenial::new(
        kind,
        format!("transition backfill stopped by execution budget: {cause:?}"),
    )
}

fn schema_denial(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        detail,
    )
}
