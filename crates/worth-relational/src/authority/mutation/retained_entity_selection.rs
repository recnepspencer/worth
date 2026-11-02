//! Apply an admitted immutable artifact, preserving the target's record history.
use super::effect_assembly::assemble_effect;
use super::outcomes::{MutationEvent, MutationOutcome, RecordMutation};
use super::{
    BranchLocalDeleteAllowance, MutationApplyDenial, MutationApplyOutcome, MutationWorkspace,
};
use crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload;
use crate::config::data::MutationConfig;
use crate::identity::data::VersionId;
use crate::runtime::{PendingRecordAllocations, PendingSymbolMeanings};
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{CommitConflict, ConflictClass};

pub(crate) fn apply(
    state: &mut WorkingState,
    selected: &PreparedRetainedEntityPayload,
    version: VersionId,
    config: &MutationConfig,
    schema: &RelationalSchemaRegistry,
    plans: &AspectContractPlanCatalog,
    symbols: &mut PendingSymbolMeanings,
    allocations: &mut PendingRecordAllocations,
    budget: Option<crate::execution::RequestWorkBudget>,
) -> Result<MutationApplyOutcome, MutationApplyDenial> {
    let mut workspace = MutationWorkspace::new(
        state,
        symbols,
        config,
        schema,
        plans,
        version,
        BranchLocalDeleteAllowance::default(),
        Some(allocations),
        budget,
    );
    let entity = selected.meaning.entity();
    let kind = selected.meaning.kind();
    let before = workspace.with_context(|context| {
        let partition = context.state.get_partition_mut(entity.partition_id);
        let old = partition
            .entity_arena
            .get(&entity)
            .filter(|slot| slot.is_live() && slot.kind_id() == Some(kind))
            .ok_or_else(stale)?;
        let before = old.extra().authoritative_aspect_state.clone();
        let mut extra = old.extra().clone();
        extra.authoritative_aspect_state = selected.payload.clone();
        partition
            .entity_arena
            .apply_extra_update(entity.slot_index(), extra, version)
            .ok_or_else(stale)?;
        context
            .state
            .mark_entity_slot_touched(entity.partition_id, entity.slot_index());
        Ok::<_, CommitConflict>(before)
    })?;
    let mut outcome = MutationOutcome::with_capacity(1, 1);
    outcome.record_change(RecordMutation::EntityUpdated {
        entity_id: entity,
        kind_id: kind,
        old_authoritative_aspect_state: before,
        new_authoritative_aspect_state: selected.payload.clone(),
        authoritative_patch: None,
    });
    outcome.record_event(MutationEvent::EntityUpdated { entity_id: entity });
    let effect = assemble_effect(outcome, &mut workspace)?;
    let preparation_telemetry = workspace.preparation_telemetry();
    let (created_entities, created_relations) = workspace.into_created_bindings();
    Ok(MutationApplyOutcome {
        effect,
        preparation_telemetry,
        created_entities,
        created_relations,
    })
}

fn stale() -> CommitConflict {
    CommitConflict::new(ConflictClass::StaleValidationBasis {
        detail: "retained selection target no longer has its admitted full-GEN entity".to_owned(),
    })
}
