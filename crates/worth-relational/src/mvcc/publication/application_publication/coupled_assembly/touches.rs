//! Append-time semantic touches and exclusive-write-set chain overlays for
//! coupled candidates (PR C §1.4–§1.5).
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::mvcc::validation::{project_applied_intents, ValidatedMutationTouch};
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::{RecordRef, TransactionCommitError};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;

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
/// created, published, or exclusively wrote; `prior_touches` are the union of
/// Native-certified touches across all earlier rounds of this chain.
#[derive(Clone)]
pub struct RelationalCoupledChainOverlay {
    head_pre_pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    exclusive_entities: Vec<EntityId>,
    prior_touches: Vec<ValidatedMutationTouch>,
}
impl RelationalCoupledChainOverlay {
    pub fn head_pre_pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.head_pre_pin
    }
    pub fn exclusive_entities(&self) -> &[EntityId] {
        &self.exclusive_entities
    }
    pub fn prior_touches(&self) -> &[ValidatedMutationTouch] {
        &self.prior_touches
    }
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Certified touches of producer part `index`'s latest applied round.
    pub fn part_append_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
        let Some(part) = self.parts.get(index) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.authored_source(),
            ));
        };
        let touches = project_applied_intents(
            execution,
            &self.mutation.state,
            &part.commands.plan().merged_intents,
        )?;
        Ok(RelationalCoupledAppendTouches {
            touches,
            records: part.records.clone(),
        })
    }

    /// Certified touches of later authored stage `stage` (0-based after prefix).
    pub fn authored_stage_append_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        stage: usize,
    ) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
        let Some(part) = self.stages.get(stage) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::UnadmittedAuthoredStage,
                self.authored_source(),
            ));
        };
        let touches = project_applied_intents(
            execution,
            &self.mutation.state,
            &part.commands.plan().merged_intents,
        )?;
        Ok(RelationalCoupledAppendTouches {
            touches,
            records: part.records.clone(),
        })
    }

    /// Verify non-intervention on chain `index`'s exclusive write set and return
    /// its head pre-pin, exclusive entities, and cumulative prior write touches.
    /// Stops with `#119 IntervenedOutput` if any later member wrote a record in
    /// this chain's write set.
    pub fn chain_overlay(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledChainOverlay, TransactionCommitError> {
        let Some(part) = self.parts.get(index) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.authored_source(),
            ));
        };
        let source = part
            .publication
            .producing_meaning()
            .expect("admitted producer route")
            .source_scope;
        let changed = &self.mutation.outcome.effect.publication.changed_records;
        execution
            .run_native_with_checkpoint(|context| {
                let own = part.records.iter().collect::<BTreeSet<_>>();
                for record in &changed[part.chain.records_end..] {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if own.contains(record) {
                        let subject = match record {
                            RecordRef::Entity(entity) => *entity,
                            RecordRef::Relation(_) => source,
                        };
                        return Err(continuation::postponed(Reason::IntervenedOutput, subject));
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0?;
        let mut prior_touches = part.chain.prior_touches.clone();
        let latest = project_applied_intents(
            execution,
            &self.mutation.state,
            &part.commands.plan().merged_intents,
        )?;
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
        Ok(RelationalCoupledChainOverlay {
            head_pre_pin: part.chain.head_input.clone(),
            exclusive_entities,
            prior_touches,
        })
    }
}
