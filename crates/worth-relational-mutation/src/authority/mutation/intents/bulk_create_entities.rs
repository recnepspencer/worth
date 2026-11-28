use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
pub mod row;
use row::apply_prepared_row;

use crate::authority::mutation::outcomes::{MutationOutcome, RecordMutation};
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::contracts::transactions::data::intents::mutation_intent::BulkEntityCreateIntent;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply<'a>(
    intent: &'a BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, TransactionCommitError> {
    let version_id = workspace.version_id();
    let mut outcome =
        MutationOutcome::with_capacity(&workspace.execution(), intent.field_patches.len(), 1)?;
    outcome.record_event(
        &workspace.execution(),
        crate::authority::mutation::outcomes::MutationEvent::BulkEntitiesCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    )?;
    workspace.run_owned_mutation(|workspace| {
        let entity_aspect_plans = stage_bulk_entity_aspect_plans(intent, workspace)?;
        for (client_key, aspect_plan) in intent.client_keys.iter().zip(entity_aspect_plans) {
            apply_prepared_row(
                intent,
                workspace,
                &mut outcome,
                version_id,
                client_key,
                aspect_plan,
            )?;
        }
        outcome.set_last_event_count(intent.field_patches.len());
        Ok(outcome)
    })
}

fn stage_bulk_entity_aspect_plans(
    intent: &BulkEntityCreateIntent,
    workspace: &MutationWorkspace<'_>,
) -> Result<
    Vec<(
        crate::authority::mutation::outcomes::MutationAspectPatch,
        worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra,
    )>,
    TransactionCommitError,
>{
    let lowered_plan = workspace.entity_aspect_plan(intent.kind_id);
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    intent
        .field_patches
        .iter()
        .map(|fields| {
            let patch = record_aspect_patch::readmit_field_authoring(
                &workspace.execution(),
                fields,
                PortablePatchReadmissionPurpose::RecordCreation,
                lowered_plan,
                target,
                FieldAuthoringDomain::Entity,
            )?;
            let state =
                record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
            Ok((patch, state))
        })
        .collect()
}
