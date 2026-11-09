//! Native data entries derived from a selected C generation through applied deltas.
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
    _key_memory: ExecutionMemoryReservation,
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
        let before_root = self.applied_before_root();
        if predecessor.source_root() != (self.runtime_instance_id(), before_root) {
            return Err(Denial::ForeignPredecessor);
        }
        crate::execution::check_request(execution).map_err(Denial::Execution)?;
        let seed = predecessor.applied_generation();
        let entries = self
            .applied_field_image()
            .expect("Native applied pin carries its field image")
            .entries(
                execution,
                (predecessor.index_id(), seed.generation_id),
                predecessor.field_locator(),
                seed,
            )
            .map_err(|cause| maintenance_denial(predecessor.index_id(), cause))?;
        let key_bytes =
            crate::storage::data::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(
                expected,
            )
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let key_memory = ExecutionMemoryReservation::reserve_in_scope(
            crate::authority::commit::pipeline::request_lease(execution),
            key_bytes,
        )
        .map_err(|cause| Denial::Execution(crate::execution::denial::lease_denial(cause)))?;
        execution
            .run_native_with_checkpoint(|context| {
                checkpoint(
                    context,
                    key_bytes.saturating_add(
                        (usize::BITS - entries.entries.len().max(1).leading_zeros()) as u64,
                    ),
                )
            })
            .map_err(|cause| Denial::Execution(cause.into()))?
            .0
            .map_err(|cause| Denial::Execution(cause.into()))?;
        let key =
            crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(expected);
        let mut read = AppliedCollectionRead {
            pin: self,
            predecessor,
            expected,
            memory: None,
            execution,
        };
        let (entities, examined, overflowed) = super::entity_field_collection::collect(
            execution,
            entries.entries.get(&key),
            predecessor.index_id(),
            predecessor.entity_kind(),
            candidate_limit,
            &mut read,
        )
        .map_err(map_stop)?;
        // Applied reads have never minted a committed-generation parity claim.
        let _ = parity;
        Ok(AppliedEntityFieldLookupOutcome {
            qualification: self.candidate_identity().qualification(),
            entities,
            examined,
            overflowed,
            _memory: read.memory,
            _key_memory: key_memory,
        })
    }
}

struct AppliedCollectionRead<'a, 'scope, 'request> {
    execution: &'a worth_execution::ExecutionRequest<'scope, 'request>,
    pin: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    predecessor: &'a PreparedEntityFieldLookup,
    expected: &'a AspectValue,
    memory: Option<ExecutionMemoryReservation>,
}
impl super::entity_field_collection::EntityFieldCollectionRead<std::convert::Infallible>
    for AppliedCollectionRead<'_, '_, '_>
{
    fn prepare_results(&mut self, examined: usize) -> Result<(), Stop<std::convert::Infallible>> {
        // Reservation is made at compare entry before collect allocates either Vec.
        let bytes = examined
            .checked_mul(2 * std::mem::size_of::<EntityId>())
            .ok_or(Stop::AccountingOverflow)?;
        self.memory = Some(
            ExecutionMemoryReservation::reserve_in_scope(
                crate::authority::commit::pipeline::request_lease(self.execution),
                bytes as u64,
            )
            .map_err(|cause| {
                lookup_execution(
                    self.predecessor.index_id(),
                    crate::execution::denial::lease_denial(cause),
                )
            })?,
        );
        Ok(())
    }
    fn compare(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        previous: usize,
    ) -> Result<Option<(crate::identity::data::KindId, bool)>, Stop<std::convert::Infallible>> {
        let index = self.predecessor.index_id();
        execution
            .run_native_with_checkpoint(|context| checkpoint(context, previous as u64 + 1))
            .map_err(|cause| lookup_execution(index, cause.into()))?
            .0
            .map_err(|cause| lookup_execution(index, cause.into()))?;
        super::field_candidate_comparison::borrowed_matches(
            execution,
            index,
            &self.pin.project_candidate(),
            entity,
            self.predecessor.field_locator(),
            self.expected,
        )
        .map(|answer| answer.map(|(_, kind, matches)| (kind, matches)))
        .map_err(Stop::Lookup)
    }
}

fn lookup_execution(
    index: crate::indexes::data::DerivedIndexId,
    cause: Cause,
) -> Stop<std::convert::Infallible> {
    Stop::Lookup(BoundedEntityFieldLookupDenial::new(
        crate::indexes::data::BoundedEntityFieldLookupDenialKind::Execution(cause),
        index,
    ))
}
fn map_stop(stop: Stop<std::convert::Infallible>) -> AppliedEntityFieldLookupDenial {
    match stop {
        Stop::Lookup(cause) => AppliedEntityFieldLookupDenial::PredecessorIndex(cause),
        Stop::Admission(never) => match never {},
        Stop::ExactBasisRequired => AppliedEntityFieldLookupDenial::ExactBasisRequired,
        Stop::AccountingOverflow => {
            AppliedEntityFieldLookupDenial::Execution(Cause::ChargedBytesOverflow)
        }
    }
}

// Data-entry preparation preserves request failures; index incoherence remains
// the existing precise, index-naming lookup refusal, not a Current fallback.
fn maintenance_denial(
    index: crate::indexes::data::DerivedIndexId,
    cause: crate::indexes::data::DerivedIndexMaintenanceDenial,
) -> AppliedEntityFieldLookupDenial {
    use crate::indexes::data::{
        BoundedEntityFieldLookupDenialKind as K, DerivedIndexMaintenanceDenialKind as M,
    };
    use AppliedEntityFieldLookupDenial as D;
    let kind = match cause.kind {
        M::Execution(cause) => return D::Execution(cause),
        M::WorkBudgetExceeded => return D::Execution(Cause::WorkExhausted),
        M::ForeignCandidate | M::BeforeRootMismatch => return D::ForeignPredecessor,
        M::Basis(_)
        | M::SnapshotUnavailable
        | M::CommitMismatch
        | M::CandidateUnavailable
        | M::CandidateLifetimeExpired { .. } => return D::ExactBasisRequired,
        M::IndexUnavailable(_) => K::IndexNotInstalled,
        M::GenerationKindMismatch(_) => K::WrongIndexKind,
        M::ColdReconstructionRequired => K::ExactGenerationUnavailable,
        M::PriorEntryMismatch
        | M::CandidateIndexesAlreadyPrepared
        | M::GenerationIdentityExhausted => K::CorruptIndexEntries,
    };
    D::PredecessorIndex(BoundedEntityFieldLookupDenial::new(kind, index))
}
