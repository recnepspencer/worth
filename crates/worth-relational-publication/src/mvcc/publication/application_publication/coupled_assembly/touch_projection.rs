//! Certified write touches and chain overlay projected from an applied member.
use super::super::custody;
use super::refusals::postponed;
use super::retained_assembly::CoupledPublicationPart;
use super::succession::guards;
use super::succession::indexes::{MemberId, MemberIndexes};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::authored_retirement_refusal::CoupledProvenancePostponement as Reason;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
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

/// The member's exact touches, plus `extra`: for a successor round, the
/// superseded round's exact touches and the kind touches of what retracting
/// it changed (#119 H-Q).
pub fn append_touches(
    execution: &ExecutionRequest<'_, '_>,
    state: &WorkingState,
    plan: &MergedCommitPlan,
    records: &[RecordRef],
    extra: &[ValidatedMutationTouch],
) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
    let mut touches = project_applied_intents(execution, state, &plan.merged_intents)?;
    if !extra.is_empty() {
        touches.extend(extra.iter().cloned());
        touches.sort_unstable();
        touches.dedup();
    }
    Ok(RelationalCoupledAppendTouches {
        touches,
        records: records.to_vec(),
    })
}

/// The chain's own-output view (#119 §3.5): its head pre-pin, every record
/// any round of it changed, its relations whose live adjacency differs, and
/// every superseded and live round's exact touches. Stops with
/// `IntervenedOutput` if a later live member changed a record of the chain.
pub fn chain_overlay(
    execution: &ExecutionRequest<'_, '_>,
    state: &WorkingState,
    part: &CoupledPublicationPart,
    indexes: &MemberIndexes,
    member: MemberId,
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
            checkpoint(context, part.chain.touched.len() as u64 + 1)
                .map_err(custody::execution_stop)
        })
        .map_err(custody::execution_stop)?
        .0?;
    if let Some(record) = guards::intervened(
        indexes,
        &part.chain.touched,
        member,
        part.chain.head_ordinal,
    ) {
        return Err(postponed(
            Reason::IntervenedOutput,
            guards::subject(record, source),
        ));
    }
    let mut prior_touches = part.chain.prior_touches.clone();
    let latest = project_applied_intents(execution, state, &part.commands.plan().merged_intents)?;
    prior_touches.extend(latest);
    prior_touches.sort_unstable();
    prior_touches.dedup();
    let exclusive_entities = part
        .chain
        .touched
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .collect::<Vec<_>>();
    let touched = part.chain.touched.iter().cloned().collect::<Vec<_>>();
    let relations = overlay_relations::overlay_relations(
        execution,
        &touched,
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
