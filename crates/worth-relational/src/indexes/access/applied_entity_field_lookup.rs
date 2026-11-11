//! Finite predecessor postings plus the actual Native applied delta.
//! No proposed index generation or Current entity identity is fabricated.
use super::PreparedEntityFieldLookup;
use crate::execution::request_stage::checkpoint;
use crate::{
    execution::RelationalExecutionDenialCause as Cause,
    identity::data::EntityId,
    indexes::data::{
        BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
        BoundedIndexParityMode,
    },
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
        RelationalPinnedRead,
    },
};
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AspectValue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppliedEntityFieldMatch {
    Absent,
    Unique(EntityId),
    Ambiguous,
}

#[derive(Debug)]
pub struct AppliedEntityFieldLookupOutcome {
    qualification: RelationalAppliedCandidateReadQualification,
    entities: Vec<EntityId>,
    overflowed: bool,
    _memory: Option<ExecutionMemoryReservation>,
    examined: usize,
}
impl AppliedEntityFieldLookupOutcome {
    pub fn qualification(&self) -> RelationalAppliedCandidateReadQualification {
        self.qualification
    }
    pub fn selection(&self) -> AppliedEntityFieldMatch {
        match self.entities.as_slice() {
            [] if !self.overflowed => AppliedEntityFieldMatch::Absent,
            [entity] if !self.overflowed => AppliedEntityFieldMatch::Unique(*entity),
            _ => AppliedEntityFieldMatch::Ambiguous,
        }
    }
    pub fn candidate_entity_ids(&self) -> &[EntityId] {
        &self.entities
    }
    pub fn into_candidate_entity_ids(self) -> Vec<EntityId> {
        self.entities
    }
    pub fn overflowed(&self) -> bool {
        self.overflowed
    }
    pub fn examined_entry_count(&self) -> usize {
        self.examined
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppliedEntityFieldLookupDenial {
    ForeignPredecessor,
    InvalidCandidateLimit,
    PredecessorIndex(BoundedEntityFieldLookupDenial),
    ExactBasisRequired,
    Execution(Cause),
}

impl RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
    pub fn lookup_applied_entity_field(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &PreparedEntityFieldLookup,
        expected: &AspectValue,
        parity: BoundedIndexParityMode,
        candidate_limit: usize,
    ) -> Result<AppliedEntityFieldLookupOutcome, AppliedEntityFieldLookupDenial> {
        use AppliedEntityFieldLookupDenial as Denial;
        if candidate_limit == 0 || candidate_limit == usize::MAX {
            return Err(Denial::InvalidCandidateLimit);
        }
        let (before_root, changed) = self.changed_entities();
        if predecessor.source_root() != (self.runtime_instance_id(), before_root) {
            return Err(Denial::ForeignPredecessor);
        }
        let limit = changed
            .len()
            .checked_add(candidate_limit)
            .filter(|n| *n < usize::MAX)
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let key_bytes =
            crate::storage::data::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(
                expected,
            )
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let slots = limit
            .checked_mul(3)
            .and_then(|n| {
                n.checked_add(
                    limit
                        .checked_add(changed.len())?
                        .checked_add(candidate_limit)?,
                )
            })
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let bytes = slots
            .checked_mul(std::mem::size_of::<EntityId>())
            .map(|n| n as u64)
            .and_then(|n| n.checked_add(key_bytes.checked_mul(2)?))
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let memory = ExecutionMemoryReservation::reserve_in_scope(
            crate::authority::commit::pipeline::request_lease(execution),
            bytes,
        )
        .map_err(|cause| Denial::Execution(crate::execution::denial::lease_denial(cause)))?;
        let old = predecessor
            .execute_prepared_entity_field_lookup(expected, limit, parity, execution, || {
                Ok::<_, std::convert::Infallible>(())
            })
            .map_err(|stop| match stop {
                Stop::Lookup(cause) => Denial::PredecessorIndex(cause),
                Stop::Admission(never) => match never {},
                Stop::ExactBasisRequired => Denial::ExactBasisRequired,
                Stop::AccountingOverflow => Denial::Execution(Cause::ChargedBytesOverflow),
            })?;
        // At most D entities changed. More than D+L predecessor matches
        // leave more than L matches even when every changed row stopped matching.
        if old.overflowed() {
            return Ok(AppliedEntityFieldLookupOutcome {
                qualification: self.candidate_identity().qualification(),
                entities: Vec::new(),
                overflowed: true,
                _memory: Some(memory),
                examined: old.examined_entry_count(),
            });
        }
        let result = execution
            .run_native_with_checkpoint(|context| {
                checkpoint(
                    context,
                    (old.candidate_entity_ids().len() + changed.len()) as u64,
                )
                .map_err(|cause| Denial::Execution(cause.into()))?;
                let mut frontier = Vec::new();
                frontier
                    .try_reserve_exact(old.candidate_entity_ids().len() + changed.len())
                    .map_err(|_| Denial::Execution(Cause::AllocationFailed))?;
                frontier.extend_from_slice(old.candidate_entity_ids());
                frontier.extend_from_slice(changed);
                let comparisons = frontier
                    .len()
                    .saturating_mul((usize::BITS - frontier.len().max(1).leading_zeros()) as usize);
                checkpoint(context, comparisons as u64)
                    .map_err(|cause| Denial::Execution(cause.into()))?;
                frontier.sort_unstable();
                frontier.dedup();
                let view = self.project_candidate();
                let mut entities = Vec::new();
                entities
                    .try_reserve_exact(candidate_limit.min(frontier.len()))
                    .map_err(|_| Denial::Execution(Cause::AllocationFailed))?;
                let mut overflowed = false;
                let mut examined = old.examined_entry_count();
                for entity in frontier {
                    checkpoint(context, key_bytes.saturating_mul(2).saturating_add(1))
                        .map_err(|cause| Denial::Execution(cause.into()))?;
                    examined += 1;
                    let matched = super::field_candidate_comparison::borrowed_matches(
                        execution,
                        predecessor.index_id(),
                        &view,
                        entity,
                        predecessor.field_locator(),
                        expected,
                    )
                    .map_err(Denial::PredecessorIndex)?;
                    if matched.is_some_and(|(_, kind, matches)| {
                        kind == predecessor.entity_kind() && matches
                    }) {
                        if entities.len() == candidate_limit {
                            overflowed = true;
                            break;
                        }
                        entities.push(entity);
                    }
                }
                Ok(AppliedEntityFieldLookupOutcome {
                    qualification: self.candidate_identity().qualification(),
                    entities,
                    overflowed,
                    _memory: Some(memory),
                    examined,
                })
            })
            .map_err(|cause| Denial::Execution(cause.into()))?;
        result.0
    }
}
