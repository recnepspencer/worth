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
use contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
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
    /// A successor round of producer part `index`, already rescoped to the
    /// coordinates its chain head chose.
    Successor {
        index: usize,
        source: contracts::identity::data::record_ids::EntityId,
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
        /// The entities the round provably deletes again (`prove_succession`).
        redeleted: &'a std::collections::BTreeSet<EntityId>,
    },
    /// The authored writer: one later stage editing an admitted stage source.
    Authored {
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
    /// The entities this suffix allocated.
    pub created:
        Vec<contracts::identity::data::record_ids::EntityId>,
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
        SuffixWriter::Authored { source, roles } => {
            local_keys::scope(execution, ordinal, created, symbols, batches, roles, naming)?;
            (
                CoupledFreshWriter::Authored,
                source,
                Writer::AuthoredPrefix,
                Default::default(),
            )
        }
    })
}

pub fn normalize(
    execution: &ExecutionRequest<'_, '_>,
    intents: OwnedNormalizedPlan,
    redeleted: Option<&std::collections::BTreeSet<EntityId>>,
    created: &CommitCreatedEntityBindings,
    aspect_plans: &AspectContractPlanCatalog,
) -> Result<
    (
        OwnedNormalizedPlan,
        Option<OwnedNormalizedPlan>,
        CoupledOriginalRelationCreates,
    ),
    TransactionCommitError,
> {
    let originals = CoupledOriginalRelationCreates::capture(
        execution,
        if redeleted.is_some() {
            &[]
        } else {
            &intents.plan().merged_intents
        },
    )?;
    let mut intents = intents.into_coupled_suffix(0, created, execution)?;
    // A successor records its commands as authored (re-creates stay
    // creates, so its touches match the one-round answer) but applies them
    // lowered onto the chain's allocations, without its proved re-deletes.
    let recorded = if let Some(redeleted) = redeleted {
        let recorded = OwnedNormalizedPlan::copy_for_request(intents.plan(), execution)?;
        intents = intents.lower_coupled_recreates(created, aspect_plans, redeleted, execution)?;
        Some(recorded)
    } else {
        None
    };
    Ok((intents, recorded, originals))
}

pub fn adopt(
    execution: &ExecutionRequest<'_, '_>,
    current_outcome: &mut MutationApplyOutcome,
    current_state: &mut WorkingState,
    outcome: MutationApplyOutcome,
    state: WorkingState,
) -> Result<(Vec<RecordRef>, Vec<EntityId>, WorkingState), TransactionCommitError> {
    let records = outcome.effect.publication.changed_records.clone();
    let created = outcome.created_entities.allocated_identities().collect();
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

    Ok((records, created, delta_state))
}
