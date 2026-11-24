//! Native data entries derived from a selected C generation through applied deltas.
//! No proposed index generation or Current entity identity is fabricated.
use crate::branch::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialOwnerPort as _, BoundedIndexParityMode,
};
use crate::branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::branch::visibility::materialization::read_records::projection::pinned::VisibilityProjectionViewOwnerPort as _;
use crate::branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use crate::branch::visibility::snapshot_states::resident_basis::ResidentVisibilitySnapshotBasis;
use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::contracts::execution::request_stage::checkpoint;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::indexes::data::applied_candidate_qualification::RelationalAppliedCandidateReadQualification;
use crate::indexes::access::prepared_entity_field_lookup::PreparedEntityFieldLookup;
use crate::indexes::authority::maintenance::applied_field::AppliedFieldImage;
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

pub struct AppliedIndexRead<'pin> {
    pub runtime_identity: u64,
    pub before_root: u64,
    pub current_version: VersionId,
    pub resident: &'pin ResidentVisibilitySnapshotBasis,
    pub fields: Option<&'pin AppliedFieldImage>,
    pub instrumentation: &'pin RuntimeInstrumentation,
    pub qualification: RelationalAppliedCandidateReadQualification,
}

pub fn lookup(
    read: &AppliedIndexRead<'_>,
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
    if predecessor.source_root() != (read.runtime_identity, read.before_root) {
        return Err(Denial::ForeignPredecessor);
    }
    crate::contracts::execution::request_custody::check_request(execution)
        .map_err(Denial::Execution)?;
    let seed = predecessor.applied_generation();
    let entries = read
        .fields
        .expect("Native applied pin carries its field image")
        .entries(
            execution,
            (predecessor.index_id(), seed.generation_id),
            predecessor.field_locator(),
            seed,
        )
        .map_err(|cause| maintenance_denial(predecessor.index_id(), cause))?;
    let key_bytes =
        crate::contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(
            expected,
        )
        .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
    let key_memory = ExecutionMemoryReservation::reserve_in_scope(
        crate::contracts::execution::request_custody::request_lease(execution),
        key_bytes,
    )
    .map_err(|cause| Denial::Execution(crate::contracts::execution::denial::lease_denial(cause)))?;
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
    let key = crate::contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey::from_aspect_value(expected);
    let mut collection = AppliedCollectionRead {
        read,
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
        &mut collection,
    )
    .map_err(map_stop)?;
    // Applied reads have never minted a committed-generation parity claim.
    let _ = parity;
    Ok(AppliedEntityFieldLookupOutcome {
        qualification: read.qualification,
        entities,
        examined,
        overflowed,
        _memory: collection.memory,
        _key_memory: key_memory,
    })
}

struct AppliedCollectionRead<'a, 'scope, 'request> {
    execution: &'a worth_execution::ExecutionRequest<'scope, 'request>,
    read: &'a AppliedIndexRead<'a>,
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
                crate::contracts::execution::request_custody::request_lease(self.execution),
                bytes as u64,
            )
            .map_err(|cause| {
                lookup_execution(
                    self.predecessor.index_id(),
                    crate::contracts::execution::denial::lease_denial(cause),
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
    ) -> Result<
        Option<(crate::contracts::identity::data::primitives::KindId, bool)>,
        Stop<std::convert::Infallible>,
    > {
        let index = self.predecessor.index_id();
        execution
            .run_native_with_checkpoint(|context| checkpoint(context, previous as u64 + 1))
            .map_err(|cause| lookup_execution(index, cause.into()))?
            .0
            .map_err(|cause| lookup_execution(index, cause.into()))?;
        super::field_candidate_comparison::borrowed_matches(
            execution,
            index,
            &VisibilityProjectionView::from_resident_basis(
                self.read.resident,
                self.read.runtime_identity,
                self.read.current_version,
                self.read.instrumentation,
            ),
            entity,
            self.predecessor.field_locator(),
            self.expected,
        )
        .map(|answer| answer.map(|(_, kind, matches)| (kind, matches)))
        .map_err(Stop::Lookup)
    }
}

fn lookup_execution(
    index: crate::contracts::indexes::data::definition::DerivedIndexId,
    cause: Cause,
) -> Stop<std::convert::Infallible> {
    Stop::Lookup(BoundedEntityFieldLookupDenial::new(
        crate::branch::indexes::data::bounded_entity_field_lookup::BoundedEntityFieldLookupDenialKind::Execution(cause),
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
    index: crate::contracts::indexes::data::definition::DerivedIndexId,
    cause: crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenial,
) -> AppliedEntityFieldLookupDenial {
    use crate::branch::indexes::data::{
        BoundedEntityFieldLookupDenialKind as K, DerivedIndexMaintenanceDenialKind as M,
    };
    use AppliedEntityFieldLookupDenial as D;
    let kind = match cause.kind {
        M::Execution(cause) => return D::Execution(cause),
        M::WorkBudgetExceeded => return D::Execution(Cause::WorkExhausted),
        M::ForeignCandidate | M::BeforeRootMismatch => return D::ForeignPredecessor,
        M::Basis(_) | M::SnapshotUnavailable | M::CommitMismatch | M::CandidateUnavailable => {
            return D::ExactBasisRequired
        }
        M::IndexUnavailable(_) => K::IndexNotInstalled,
        M::GenerationKindMismatch(_) => K::WrongIndexKind,
        M::ColdReconstructionRequired => K::ExactGenerationUnavailable,
        M::PriorEntryMismatch
        | M::CandidateIndexesAlreadyPrepared
        | M::GenerationIdentityExhausted => K::CorruptIndexEntries,
    };
    D::PredecessorIndex(BoundedEntityFieldLookupDenial::new(kind, index))
}
