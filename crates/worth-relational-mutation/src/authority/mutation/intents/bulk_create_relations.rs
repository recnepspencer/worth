use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};

use crate::authority::mutation::outcomes::{MutationOutcome, RecordMutation};
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::contracts::transactions::data::intents::mutation_intent::BulkRelationCreateIntent;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::EntityReference;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub mod row;
use row::apply_relation_row;

pub(super) fn apply<'a>(
    intent: &'a BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, TransactionCommitError> {
    let version_id = workspace.version_id();
    let mut outcome =
        MutationOutcome::with_capacity(&workspace.execution(), intent.endpoints.len(), 1)?;
    outcome.record_event(
        &workspace.execution(),
        crate::authority::mutation::outcomes::MutationEvent::BulkRelationsCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    )?;
    workspace.run_owned_mutation(|workspace| {
        let empty_fields = AspectFieldPatch::default();
        for (row_index, (source, target)) in intent.endpoints.iter().enumerate() {
            let fields = intent.field_patches.get(row_index).unwrap_or(&empty_fields);
            apply_relation_row(
                intent,
                workspace,
                &mut outcome,
                version_id,
                row_index,
                source,
                target,
                fields,
            )?;
        }
        Ok::<_, TransactionCommitError>(())
    })?;
    outcome.set_last_event_count(intent.endpoints.len());
    Ok(outcome)
}

fn resolve_entity_reference(
    workspace: &MutationWorkspace<'_>,
    entity_reference: &EntityReference,
) -> Result<crate::contracts::identity::data::record_ids::EntityId, CommitConflict> {
    workspace
        .resolve_entity_reference(entity_reference)
        .ok_or_else(|| {
            CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
                detail:
                    "relation endpoints must resolve within the same authoritative commit scope"
                        .to_string(),
            })
        })
}
