use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_durability::internal_port::durability::migration::readmitted_canonical_commit::ReadmittedCanonicalCommitOwnerPort as _;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::CanonicalCommitAuthorityKind;
use crate::runtime::RelationalRuntime;
use std::collections::BTreeSet;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{CanonicalCommitEnvelopeBodyFieldsPort as _ };

use super::root_inventory::RecoveredRootInventory;

pub(crate) use worth_relational_durability::internal_port::durability::authority::runtime_rebuild::envelope_replay::{commit_stop, naming_recovery, path, preparation};
mod committed_artifacts;
mod effect;
mod owner_bindings;

#[cfg(test)]
mod interruption_tests;

use effect::{replay_envelope_effect, restore_authoritative_artifacts_when_required};
use path::validate_replay_path;
use preparation::{prepare_replay_context, ReplayPreparationInputs};

pub(super) fn replay_readmitted_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    readmitted: crate::durability::migration::ReadmittedCanonicalCommit,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
    restore_authoritative_envelope: bool,
    recovered_roots: &mut RecoveredRootInventory,
    recorded_result_schema: Option<crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema>,
) -> Result<crate::history::data::PositionedCanonicalCommit, RelationalRecoveryStop> {
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        let envelope = readmitted.envelope();
        validate_replay_path(envelope)?;
        let catalog = super::index_catalog::readmit(
            execution,
            &restored.indexes,
            restored.runtime_instance_id(),
            &readmitted,
            || {
                restored
                    .index_authority()
                    .recover_installed_index_routing(execution)
            },
        )?;
        for (_, text) in &envelope.owner_symbol_texts().entries {
            restored
                .services
                .symbols
                .demand_text(execution, text.as_str())?;
        }
        restored
            .services
            .symbols
            .import_canonical_snapshot(envelope.owner_symbol_texts())
            .map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptCanonicalArtifact, detail)
            })?;
        let had_checkpoint_artifact = prepare_replay_context(
            execution,
            ReplayPreparationInputs {
                history: &restored.history,
                lineage: &restored.lineage,
                registry: &restored.config.schema.registry,
                symbols: &restored.services.symbols,
            },
            envelope,
            available_commit_ids,
            recovered_roots,
            |nodes, parents| {
                restored
                    .performance_access()
                    .count_merge_history_durability_validation(nodes, parents);
            },
        )?;
        restored.history.advance_canonical_stream_floor(
            crate::publication::patch::data::PatchStreamPosition(
                readmitted.position().0.saturating_sub(1),
            ),
        );
        let positioned = readmitted.positioned().clone();
        let replays_mutation_pipeline =
            envelope.authority_kind() == CanonicalCommitAuthorityKind::VersionedTransaction;
        if !replays_mutation_pipeline
            && (!envelope.record_allocations().is_empty()
                || !envelope.cancelled_record_allocations().is_empty())
        {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "durable commit {} carries record allocation evidence without a mutation replay path",
                    envelope.commit.commit_id.0
                ),
            ).into());
        }
        if replays_mutation_pipeline {
            restored
                .record_identity
                .stage_replay_allocations(
                    envelope.record_allocations().to_vec(),
                    envelope.cancelled_record_allocations().to_vec(),
                )
                .map_err(|denial| {
                    DurabilityError::new(RecoveryFailureClass::ReplayFailure, denial.detail())
                })?;
        }
        let replay_result = replay_envelope_effect(
            lane,
            execution,
            restored,
            &positioned,
            catalog,
            recorded_result_schema,
        );
        let staged_was_not_consumed = restored.record_identity.clear_staged_replay_allocations();
        replay_result?;
        if staged_was_not_consumed && replays_mutation_pipeline {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "durable commit {} did not consume its canonical record allocation evidence",
                    envelope.commit.commit_id.0
                ),
            )
            .into());
        }
        restore_authoritative_artifacts_when_required(
            execution,
            restored,
            &positioned,
            restore_authoritative_envelope,
            !had_checkpoint_artifact,
        )?;
        Ok(positioned)
    })
}
