//! Certified write touches and chain overlay projected from an applied member.
use super::super::custody;
use super::refusals::postponed;
use super::retained_assembly::CoupledPublicationPart;
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::authored_retirement_refusal::CoupledProvenancePostponement as Reason;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::{
    project_applied_intents, ValidatedMutationTouch,
};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

mod overlay_relations;
pub use overlay_relations::{RelationalCoupledOverlayEndpoints, RelationalCoupledOverlayRelation};

/// Native-certified touches projected from one applied coupled member.
#[derive(Clone, Debug)]
pub struct RelationalCoupledAppendTouches {
    touches: Vec<ValidatedMutationTouch>,
    records: Vec<RecordRef>,
}
impl RelationalCoupledAppendTouches {
    pub fn touches(&self) -> &[ValidatedMutationTouch] {
        &self.touches
    }
    pub fn records(&self) -> &[RecordRef] {
        &self.records
    }
}

/// Native-certified exclusive-write-set overlay for reopening a coupled chain.
/// `head_pre_pin` is the candidate image immediately before the chain's head
/// appended; `exclusive_entities` are the entities the chain's earlier rounds
/// created, published, or exclusively wrote; `relations` are the chain's
/// relation records whose live adjacency differs between the head pre-pin and
/// the current candidate (#277); `prior_touches` are the union of
/// Native-certified touches across all earlier rounds of this chain.
#[derive(Clone)]
pub struct RelationalCoupledChainOverlay {
    head_pre_pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    exclusive_entities: Vec<EntityId>,
    relations: Vec<RelationalCoupledOverlayRelation>,
    prior_touches: Vec<ValidatedMutationTouch>,
}
impl RelationalCoupledChainOverlay {
    pub fn head_pre_pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.head_pre_pin
    }
    pub fn exclusive_entities(&self) -> &[EntityId] {
        &self.exclusive_entities
    }
    /// The chain's relations with changed live adjacency, sorted by id. Each is
    /// exclusive to the chain: a later member's write to one stops the round
    /// with `IntervenedOutput` before this overlay exists.
    pub fn relations(&self) -> &[RelationalCoupledOverlayRelation] {
        &self.relations
    }
    pub fn prior_touches(&self) -> &[ValidatedMutationTouch] {
        &self.prior_touches
    }
}

pub fn append_touches(
    execution: &ExecutionRequest<'_, '_>,
    state: &WorkingState,
    plan: &MergedCommitPlan,
    records: &[RecordRef],
) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
    let touches = project_applied_intents(execution, state, &plan.merged_intents)?;
    Ok(RelationalCoupledAppendTouches {
        touches,
        records: records.to_vec(),
    })
}

pub fn chain_overlay(
    execution: &ExecutionRequest<'_, '_>,
    state: &WorkingState,
    changed: &[RecordRef],
    part: &CoupledPublicationPart,
    current: impl FnOnce() -> Result<
        RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        TransactionCommitError,
    >,
) -> Result<RelationalCoupledChainOverlay, TransactionCommitError> {
    let source = part
        .publication
        .producing_meaning()
        .expect("admitted producer route")
        .source_scope;
    execution
        .run_native_with_checkpoint(|context| {
            let own = part.records.iter().collect::<BTreeSet<_>>();
            let mut charge = |units| checkpoint(context, units).map_err(custody::execution_stop);
            if let Some(record) = part.chain.intervened(changed, &own, &mut charge)? {
                let subject = match record {
                    RecordRef::Entity(entity) => *entity,
                    RecordRef::Relation(_) => source,
                };
                return Err(postponed(Reason::IntervenedOutput, subject));
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    let mut prior_touches = part.chain.prior_touches.clone();
    let latest = project_applied_intents(execution, state, &part.commands.plan().merged_intents)?;
    prior_touches.extend(latest);
    prior_touches.sort_unstable();
    prior_touches.dedup();
    let mut exclusive_entities = part
        .records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .collect::<Vec<_>>();
    exclusive_entities.sort_unstable();
    exclusive_entities.dedup();
    let relations = overlay_relations::overlay_relations(
        execution,
        &part.records,
        &part.chain.head_input,
        &current()?,
    )?;
    Ok(RelationalCoupledChainOverlay {
        head_pre_pin: part.chain.head_input.clone(),
        exclusive_entities,
        relations,
        prior_touches,
    })
}
