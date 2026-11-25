//! Lower declarative field defaults into atomic native entity backfill intents
//! under cooperative work metering and strict pre-allocation retained-memory
//! admission using fallible `Vec::try_reserve_exact` and unstable sorting.

use std::collections::BTreeMap;
use std::sync::Mutex;

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
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_relational::facade::{
    identity::EntityId,
    transactions::{
        ApplyEntityAspectPatchIntent, AspectFieldPatch, EntityMutationIntent, MutationIntent,
        UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

use super::backfill_memory::{
    batch_allocation_bytes, charge_work, contract_group_retained_bytes, count_batch_intents,
    count_distinct_runs, entity_backfill_retained_bytes, planned_locator_and_value_bytes,
    reserve_retained, retained_bytes_overflow, schema_denial, vec_backing_bytes,
    work_ceiling_denial,
};

const BATCH_NAME_PREFIX: &str = "transition-default-backfill:";

pub(super) struct BackfillPlan {
    pub(super) batches: Vec<WorkerIntentBatch>,
    pub(super) backfilled_records: usize,
}

pub(super) struct AspectDefaultGroup {
    pub(super) aspect_key: AspectKey,
    pub(super) contract: AspectContract,
    pub(super) locators: Vec<(AspectFieldLocator, AspectValue)>,
}

struct BackfillRequest<'a, 'b, 'c> {
    descriptor: &'a WorthQueryProgramTransitionDescriptor,
    max_entities_per_kind: usize,
    retained_memory: &'a Mutex<Vec<ExecutionMemoryReservation>>,
    source: &'a WorthQueryHomeProgramTransitionReadSource<'b, 'c>,
    target: &'a WorthQueryHomeProgramTransitionTarget<'a>,
}

pub(super) fn build_default_backfill_batches(
    descriptor: &WorthQueryProgramTransitionDescriptor,
    max_entities_per_kind: usize,
    retained_memory: &Mutex<Vec<ExecutionMemoryReservation>>,
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<BackfillPlan, WorthQueryPrimaryGraphInstallationDenial> {
    let request = source.execution_request();
    let backfill_req = BackfillRequest {
        descriptor,
        max_entities_per_kind,
        retained_memory,
        source,
        target,
    };
    let mut inner: Result<BackfillPlan, WorthQueryPrimaryGraphInstallationDenial> =
        Ok(BackfillPlan {
            batches: Vec::new(),
            backfilled_records: 0,
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
    let raw_defaults = input.descriptor.added_defaults();
    if raw_defaults.is_empty() {
        return Ok(BackfillPlan {
            batches: Vec::new(),
            backfilled_records: 0,
        });
    }
    reserve_retained(
        request,
        context,
        input.retained_memory,
        vec_backing_bytes::<&WorthQueryProgramTransitionFieldDefault>(raw_defaults.len())?,
    )?;
    let mut ordered_defaults: Vec<&WorthQueryProgramTransitionFieldDefault> = Vec::new();
    ordered_defaults
        .try_reserve_exact(raw_defaults.len())
        .map_err(|_| retained_bytes_overflow())?;
    for field_default in raw_defaults {
        charge_work(context, 1)?;
        ordered_defaults.push(field_default);
    }
    ordered_defaults.sort_unstable_by_key(|d| (d.entity(), d.aspect(), d.field()));

    let entity_count = count_distinct_runs(&ordered_defaults, |d| d.entity());
    reserve_retained(
        request,
        context,
        input.retained_memory,
        vec_backing_bytes::<WorkerIntentBatch>(entity_count)?,
    )?;
    let mut batches = Vec::new();
    batches
        .try_reserve_exact(entity_count)
        .map_err(|_| retained_bytes_overflow())?;
    let mut backfilled_records = 0usize;

    let mut cursor = 0usize;
    while cursor < ordered_defaults.len() {
        let entity_name = ordered_defaults[cursor].entity();
        let mut end = cursor + 1;
        while end < ordered_defaults.len() && ordered_defaults[end].entity() == entity_name {
            end += 1;
        }
        let defaults = &ordered_defaults[cursor..end];
        cursor = end;

        charge_work(context, 1)?;
        let groups = resolve_aspect_groups(
            entity_name,
            defaults,
            input.target,
            request,
            context,
            input.retained_memory,
        )?;
        let rows = input
            .source
            .entities(entity_name, input.max_entities_per_kind)?;
        if rows.records().is_empty() {
            continue;
        }

        let transition_name = input.descriptor.transition_name();
        let name_len = BATCH_NAME_PREFIX
            .len()
            .checked_add(transition_name.len())
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_add(entity_name.len()))
            .ok_or_else(retained_bytes_overflow)?;
        let exact_intents = count_batch_intents(
            rows.records()
                .iter()
                .map(|row| row.authoritative_aspect_state.as_ref()),
            &groups,
        )?;
        let header_bytes = batch_allocation_bytes(name_len, exact_intents)?;
        reserve_retained(request, context, input.retained_memory, header_bytes)?;

        let mut batch_name = String::new();
        batch_name
            .try_reserve_exact(name_len)
            .map_err(|_| retained_bytes_overflow())?;
        batch_name.push_str(BATCH_NAME_PREFIX);
        batch_name.push_str(transition_name);
        batch_name.push(':');
        batch_name.push_str(entity_name);
        let mut batch = WorkerIntentBatch::new(batch_name);
        batch
            .intents
            .try_reserve_exact(exact_intents)
            .map_err(|_| retained_bytes_overflow())?;

        for row in rows.records() {
            let field_count = defaults.len().max(1) as u64;
            charge_work(context, field_count)?;
            let row_bytes =
                entity_backfill_retained_bytes(row.authoritative_aspect_state.as_ref(), &groups)?;
            reserve_retained(request, context, input.retained_memory, row_bytes)?;
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

    Ok(BackfillPlan {
        batches,
        backfilled_records,
    })
}

fn resolve_aspect_groups(
    entity_name: &'static str,
    defaults: &[&WorthQueryProgramTransitionFieldDefault],
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
    request: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    retained_memory: &Mutex<Vec<ExecutionMemoryReservation>>,
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

    let aspect_count = count_distinct_runs(defaults, |d| d.aspect());
    reserve_retained(
        request,
        context,
        retained_memory,
        vec_backing_bytes::<AspectDefaultGroup>(aspect_count)?,
    )?;
    let mut groups = Vec::new();
    groups
        .try_reserve_exact(aspect_count)
        .map_err(|_| retained_bytes_overflow())?;

    let mut cursor = 0usize;
    while cursor < defaults.len() {
        let aspect_name = defaults[cursor].aspect();
        let mut end = cursor + 1;
        while end < defaults.len() && defaults[end].aspect() == aspect_name {
            end += 1;
        }
        let aspect_slice = &defaults[cursor..end];
        cursor = end;

        charge_work(context, 1)?;
        let binding = registration
            .aspect_contract_declarations
            .aspects
            .iter()
            .find(|declared| declared.aspect_key().as_str() == aspect_name)
            .ok_or_else(|| {
                schema_denial(format!(
                    "target aspect `{aspect_name}` on `{entity_name}` has no declared contract"
                ))
            })?;

        let aspect_key = binding.aspect_key();
        let group_overhead =
            contract_group_retained_bytes(&aspect_key, &binding.contract, aspect_slice.len())?;
        reserve_retained(request, context, retained_memory, group_overhead)?;

        let mut locators: Vec<(AspectFieldLocator, AspectValue)> = Vec::new();
        locators
            .try_reserve_exact(aspect_slice.len())
            .map_err(|_| retained_bytes_overflow())?;
        for field_default in aspect_slice {
            charge_work(context, 1)?;
            let entry_bytes = planned_locator_and_value_bytes(
                field_default.aspect(),
                field_default.field(),
                field_default.default_value(),
            )?;
            reserve_retained(request, context, retained_memory, entry_bytes)?;
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
            locators.push((locator, field_default.default_value().clone()));
        }
        locators.sort_unstable_by(|left, right| left.0.cmp(&right.0));
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
    let attach_count = groups
        .iter()
        .filter(|group| !aspect_state.is_some_and(|state| state.get(&group.aspect_key).is_some()))
        .count();
    let mut existing_aspect_fields: BTreeMap<AspectFieldLocator, AspectValue> = BTreeMap::new();
    let mut attach_ops: Vec<PortableAspectPatchOperation> = Vec::new();
    attach_ops
        .try_reserve_exact(attach_count)
        .map_err(|_| retained_bytes_overflow())?;

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
        let patch = ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch: PortableRecordAspectPatch::new(attach_ops),
        };
        batch = batch.push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(patch),
        ));
    }
    if !existing_aspect_fields.is_empty() {
        let update = UpdateEntityFieldsIntent {
            entity_id,
            fields: AspectFieldPatch::new(existing_aspect_fields),
        };
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            update,
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
            let mut entries: Vec<(FieldKey, AspectValue)> = Vec::new();
            entries
                .try_reserve_exact(group.locators.len())
                .map_err(|_| retained_bytes_overflow())?;
            for (locator, field_value) in &group.locators {
                let [field_key] = locator.field_path().fields() else {
                    return Err(schema_denial(format!(
                        "nested struct default path `{:?}` is not supported for aspect attachment",
                        locator.field_path()
                    )));
                };
                entries.push((field_key.clone(), field_value.clone()));
            }
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            let struct_value = StructAspectValue::from_sorted_fields(entries.into_boxed_slice())
                .map_err(|cause| {
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
            let (_, scalar) = &group.locators[0];
            ContractValidationInput::Scalar(scalar.clone())
        }
    };
    Ok(PortableAspectPatchOperation::SetWhole { basis, value })
}
