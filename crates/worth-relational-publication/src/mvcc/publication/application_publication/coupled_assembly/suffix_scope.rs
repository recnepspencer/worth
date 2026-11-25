//! Local writer custody, command normalization and sparse suffix adoption.
use worth_relational_contracts::internal_port as contracts;
use super::super::custody;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use super::{local_keys, writer_custody::Writer};
use worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use contracts::identity::data::record_ids::EntityId;
use worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use contracts::publication::data::authored_retirement_refusal::CoupledFreshWriter;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use contracts::symbols::data::string_interner::StringInterner;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::created_relation_bindings::coupled::CoupledOriginalRelationCreates;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use worth_execution::ExecutionRequest;

/// Who writes this suffix, and under which custody.
pub enum SuffixWriter<'a> {
    /// An admitted producer route, `index` in the producer roster.
    Producer {
        index: usize,
        source: contracts::identity::data::record_ids::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
    },
    /// A successor round of producer part `index`, already rescoped to its
    /// chain's coordinates and applied as written over the retracted image.
    Successor {
        index: usize,
        source: contracts::identity::data::record_ids::EntityId,
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
    },
    /// The authored writer: later stage `stage` editing an admitted stage source.
    Authored {
        stage: usize,
        source: contracts::identity::data::record_ids::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
    },
}

pub struct AppliedSuffix {
    pub read:
        RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub plan:
        worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub records:
        Vec<RecordRef>,
    pub content:
        worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub delta_state:
        WorkingState,
    pub local_keys:
        local_keys::LocalKeys,
    /// The records this suffix created, by source and key text.
    pub creates:
        Vec<super::succession::ledger::ChainCreate>,
    /// Whether this suffix deleted an entity.
    pub has_entity_deletions: bool,
}

pub fn scope<'a>(
    execution: &ExecutionRequest<'_, '_>,
    writer: SuffixWriter<'a>,
    ordinal: usize,
    created: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    naming: Option<&mut contracts::lineage::data::CommitNamingPlan>,
) -> Result<
    (
        CoupledFreshWriter,
        EntityId,
        Writer<'a>,
        local_keys::LocalKeys,
    ),
    TransactionCommitError,
> {
    Ok(match writer {
        SuffixWriter::Producer {
            index,
            source,
            roles,
            predecessor,
        } => {
            let keys =
                local_keys::scope(execution, ordinal, created, symbols, batches, roles, naming)?;
            (
                CoupledFreshWriter::Producer(index),
                source,
                Writer::Producer(predecessor),
                keys,
            )
        }
        SuffixWriter::Successor {
            index,
            source,
            predecessor,
            ..
        } => (
            CoupledFreshWriter::Producer(index),
            source,
            Writer::Producer(predecessor),
            Default::default(),
        ),
        SuffixWriter::Authored {
            stage,
            source,
            roles,
        } => {
            local_keys::scope(execution, ordinal, created, symbols, batches, roles, naming)?;
            (
                CoupledFreshWriter::Authored,
                source,
                Writer::AuthoredStage(stage),
                Default::default(),
            )
        }
    })
}

pub fn normalize(
    execution: &ExecutionRequest<'_, '_>,
    intents: OwnedNormalizedPlan,
    created: &CommitCreatedEntityBindings,
) -> Result<(OwnedNormalizedPlan, CoupledOriginalRelationCreates), TransactionCommitError> {
    let originals =
        CoupledOriginalRelationCreates::capture(execution, &intents.plan().merged_intents)?;
    let intents = intents.into_coupled_suffix(0, created, execution)?;
    Ok((intents, originals))
}

/// Adopt one applied suffix into the candidate. Returns its changed records,
/// its creates (for its chain's ledger) and its sparse state.
pub fn adopt(
    execution: &ExecutionRequest<'_, '_>,
    current_outcome: &mut MutationApplyOutcome,
    current_state: &mut WorkingState,
    outcome: MutationApplyOutcome,
    state: WorkingState,
    symbols: &StringInterner,
) -> Result<
    (
        Vec<RecordRef>,
        Vec<super::succession::ledger::ChainCreate>,
        WorkingState,
    ),
    TransactionCommitError,
> {
    use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_source::AllocationSource;
    let records = outcome.effect.publication.changed_records.clone();
    let text =
        |key: &contracts::symbols::data::client_key::ClientKey| {
            key.resolve_with_interner(symbols).map(str::to_owned).ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "coupled local key has no actual Native symbol meaning",
            )
        })
        };
    let mut creates = Vec::new();
    for (created, id) in outcome.created_entities.iter() {
        creates.push(super::succession::ledger::ChainCreate {
            source: AllocationSource::Entity {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
            },
            record: RecordRef::Entity(*id),
            text: text(&created.client_key)?,
        });
    }
    for (created, id) in outcome.created_relations.iter() {
        creates.push(super::succession::ledger::ChainCreate {
            source: AllocationSource::Relation {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
            },
            record: RecordRef::Relation(*id),
            text: text(&created.client_key)?,
        });
    }
    current_outcome
        .created_entities
        .append_owned(outcome.created_entities)
        .map_err(TransactionCommitError::conflict)?;
    current_outcome
        .created_relations
        .append_owned(outcome.created_relations)
        .map_err(TransactionCommitError::conflict)?;
    current_outcome
        .preparation_telemetry
        .accumulate(outcome.preparation_telemetry);
    current_outcome.effect.accumulate(outcome.effect);
    // Retain only this part's actual sparse state for the next physical pin.
    // Applying it over the previous D reuses the ordinary root-region owner.
    let delta_state = state.clone();
    current_state
        .adopt_sequential_apply(state, execution)
        .map_err(custody::execution_stop)?;

    Ok((records, creates, delta_state))
}
