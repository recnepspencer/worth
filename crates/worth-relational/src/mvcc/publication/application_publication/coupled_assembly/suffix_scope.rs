//! Local writer custody, command normalization and sparse suffix adoption.
use super::super::custody;
use super::super::RequiredOutputRoleTarget;
use super::{local_keys, writer_custody::Writer};
use crate::authority::mutation::MutationApplyOutcome;
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::identity::data::EntityId;
use crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use crate::publication::data::CoupledFreshWriter;
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::WorkingState;
use crate::symbols::data::StringInterner;
use crate::transactions::data::outcomes::{
    created_entity_bindings::CommitCreatedEntityBindings,
    created_relation_bindings::CoupledOriginalRelationCreates,
    transaction_commit_error::TransactionCommitError,
};
use crate::transactions::data::primitives::{RecordRef, WorkerIntentBatch};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use worth_execution::ExecutionRequest;

/// Who writes this suffix, and under which custody.
pub(in crate::mvcc::publication::application_publication::coupled_assembly) enum SuffixWriter<'a> {
    /// An admitted producer route, `index` in the producer roster.
    Producer {
        index: usize,
        source: crate::identity::data::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
    },
    /// A successor round of producer part `index`, already rescoped to the
    /// coordinates its chain head chose.
    Successor {
        index: usize,
        source: crate::identity::data::EntityId,
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
    },
    /// The authored writer: one later stage editing an admitted stage source.
    Authored {
        source: crate::identity::data::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
    },
}

pub(in crate::mvcc::publication::application_publication::coupled_assembly) struct AppliedSuffix {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) read:
        RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) plan:
        crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) records:
        Vec<RecordRef>,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) content:
        crate::content::publication::ContentPublication,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) delta_state:
        WorkingState,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) local_keys:
        local_keys::LocalKeys,
    /// The entities this suffix allocated.
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) created:
        Vec<crate::identity::data::EntityId>,
}

pub(super) fn scope<'a>(
    execution: &ExecutionRequest<'_, '_>,
    writer: SuffixWriter<'a>,
    ordinal: usize,
    created: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
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
            let keys = local_keys::scope(execution, ordinal, created, symbols, batches, roles)?;
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
        } => (
            CoupledFreshWriter::Producer(index),
            source,
            Writer::Producer(predecessor),
            Default::default(),
        ),
        SuffixWriter::Authored { source, roles } => {
            local_keys::scope(execution, ordinal, created, symbols, batches, roles)?;
            (
                CoupledFreshWriter::Authored,
                source,
                Writer::AuthoredPrefix,
                Default::default(),
            )
        }
    })
}

pub(super) fn normalize(
    execution: &ExecutionRequest<'_, '_>,
    intents: OwnedNormalizedPlan,
    successor: bool,
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
        if successor {
            &[]
        } else {
            &intents.plan().merged_intents
        },
    )?;
    let mut intents = intents.into_coupled_suffix(0, created, execution)?;
    // A successor records its commands as authored (re-creates stay
    // creates, so its touches match the one-round answer) but applies them
    // lowered onto the chain's allocations.
    let recorded = if successor {
        let recorded = OwnedNormalizedPlan::copy_for_request(intents.plan(), execution)?;
        intents = intents.lower_coupled_recreates(created, aspect_plans, execution)?;
        Some(recorded)
    } else {
        None
    };
    Ok((intents, recorded, originals))
}

pub(super) fn adopt(
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
