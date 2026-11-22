use std::sync::Arc;
use std::sync::OnceLock;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;

use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;

pub mod equivalence;
pub mod pending_dependency;
pub mod publication_recovery;
pub use publication_recovery::PublicationRecoveryStop;
pub mod rebound;
pub mod requested_read;
pub mod test_support;
pub mod verification;

pub use verification::map_admission_stop;
pub use verification::map_verification_stop;
pub use verification::reserve_pending;
pub use verification::ConsumedOutputVerification;
pub use verification::ConsumedOutputVerificationStop;
pub use verification::EvidenceView;
pub use verification::VerificationCapacityStop;

/// One output actually selected by a projected read. The lineage owner minted
/// the identity; Arc only shares retained evidence and is not an identity.
/// This runtime-only record is never serialized through a TypeId-bearing key.
pub mod applied;
pub use applied::AppliedConsumedOutput;

/// Recorded input custody and an attempt-local appended prerequisite are distinct.
/// Only the sole final publication may fill the appended prerequisite's cell.
#[derive(Clone, Debug)]
pub enum ConsumedOutputEvidence {
    Recorded(RecordedConsumedOutput),
    Appended(AppliedConsumedOutput),
}
#[derive(Clone, Debug)]
pub struct RecordedConsumedOutput {
    source: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence,
    identity: Arc<RecordedSettlementIdentity>,
    source_facts:
        Option<crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts>,
    upstream: Arc<[ConsumedOutputEvidence]>,
    verification_requirement: Option<FullVerificationReason>,
    native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
    selected_native_root: ConsumedReadRoot,
}
#[derive(Clone, Debug)]
enum ConsumedReadRoot {
    Selected(Arc<PositionedRelationalSnapshot>),
    Published(PositionedRelationalSnapshot),
}
impl ConsumedReadRoot {
    fn as_ref(&self) -> &PositionedRelationalSnapshot {
        match self {
            Self::Selected(root) => root,
            Self::Published(root) => root,
        }
    }
}
impl std::ops::Deref for ConsumedReadRoot {
    type Target = PositionedRelationalSnapshot;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl PartialEq for ConsumedReadRoot {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for ConsumedReadRoot {}

impl PartialEq for ConsumedOutputEvidence {
    fn eq(&self, other: &Self) -> bool {
        match (self.applied(), other.applied()) {
            (Some(left), Some(right)) => left.same_selection(right),
            (None, None) => self.recorded() == other.recorded(),
            _ => false,
        }
    }
}

impl Eq for ConsumedOutputEvidence {}

impl PartialEq for RecordedConsumedOutput {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.source == other.source
            && self.source_facts == other.source_facts
            && self.upstream == other.upstream
            && self.verification_requirement == other.verification_requirement
            && match (&self.native_output_witness, &other.native_output_witness) {
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
            && self.selected_native_root == other.selected_native_root
    }
}

impl Eq for RecordedConsumedOutput {}

impl ConsumedOutputEvidence {
    pub fn applied(&self) -> Option<&AppliedConsumedOutput> {
        match self {
            Self::Appended(link) if !link.is_settled() => Some(link),
            Self::Appended(_) | Self::Recorded(_) => None,
        }
    }
    fn recorded(&self) -> &RecordedConsumedOutput {
        match self {
            Self::Recorded(recorded) => recorded,
            Self::Appended(link) => link.recorded(),
        }
    }

    /// The exact native basis on which the read sealed this current input.
    pub fn current_read_basis(&self) -> (u64, u64) {
        (
            self.recorded().selected_native_root.runtime_instance_id(),
            self.recorded().selected_native_root.root_id(),
        )
    }

    pub fn new(
        computation: crate::domain_computation::primary_graph::output_lineage::CurrentComputation,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: Arc<[ConsumedOutputEvidence]>,
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: Arc<OnceLock<SealedNativeOutputWitness>>,
        selected_native_root: Arc<PositionedRelationalSnapshot>,
    ) -> Self {
        Self::Recorded(RecordedConsumedOutput {
            source: computation.source(),
            identity,
            source_facts: Some(source_facts),
            upstream,
            verification_requirement,
            native_output_witness: Some(native_output_witness),
            selected_native_root: ConsumedReadRoot::Selected(selected_native_root),
        })
    }

    /// Final publication retains genuine settled provenance without asserting
    /// that the performed inputs certify Current. Verification checks that state.
    pub fn settled(
        source: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: Option<
            crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        >,
        upstream: Arc<[Self]>,
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
        selected_native_root: PositionedRelationalSnapshot,
    ) -> Self {
        Self::Recorded(RecordedConsumedOutput {
            source,
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness,
            selected_native_root: ConsumedReadRoot::Published(selected_native_root),
        })
    }

    pub fn identity(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.recorded().identity
    }

    /// A consumed restored output gets its mark row at the basis this reader
    /// compared it in full; any other output answers for its own row. Returns
    /// whether a consumer may register over this output.
    pub fn establish_restored(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<bool, SettlementRegistrationStop> {
        if self.recorded().verification_requirement
            != Some(FullVerificationReason::CheckpointRestore)
        {
            return Ok(true);
        }
        let witness = self.recorded().native_output_witness.as_ref();
        let Some(witness) = witness.and_then(|witness| witness.get()) else {
            return Ok(false);
        };
        if !self.recorded().upstream.is_empty() {
            return Ok(false);
        }
        owner.establish_verified_root(
            &self.recorded().selected_native_root,
            &self.recorded().identity,
            self.recorded()
                .source_facts
                .as_ref()
                .ok_or(SettlementRegistrationStop::SourceUnavailable)?,
            witness,
        )
    }

    /// Only the recovered root actually verified by a native reader may be
    /// retained without an executable source row. This is static settlement
    /// custody, never permission to reconstruct or refresh a source query.
    pub fn claim_verified_checkpoint_root(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<bool, SettlementRegistrationStop> {
        if self.recorded().verification_requirement
            != Some(FullVerificationReason::CheckpointRestore)
            || !self.recorded().upstream.is_empty()
            || self
                .recorded()
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get())
                .is_none()
            || !self.establish_restored(owner)?
        {
            return Ok(false);
        }
        Ok(matches!(
            owner.currentness(&self.recorded().selected_native_root, &self.recorded().identity)?,
            crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness::Clean
        ))
    }

    /// Record the restored root on the commit's meter. The read basis supplies
    /// only a branch identity to minting; Native selects its true head. A
    /// contended Native publication or changed lookup declines this derived
    /// registration, leaving the commit to compare the root in full again.
    pub fn establish_restored_before_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_owner: &WorthQueryRelationalSourceOwner,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        if self.recorded().verification_requirement
            != Some(FullVerificationReason::CheckpointRestore)
        {
            return Ok(());
        }
        use crate::domain_computation::primary_graph::output_lineage::invalidation::HeadCellRegistrationStop;
        use worth_query_execution_runtime::facade::ports::RelationalSourceOwnerInternalPort as _;
        match source_owner
            .mint_mark_cell_at_head(execution, self.recorded().selected_native_root.branch_id())
        {
            Ok(()) => {}
            Err(HeadCellRegistrationStop::Admission(stop)) => return Err(map_admission_stop(stop)),
            Err(HeadCellRegistrationStop::Native(stop)) => {
                use worth_relational::facade::mvcc::PublicationCompanionRegistrationStop;
                return match stop {
                    PublicationCompanionRegistrationStop::HeadCellPublicationContended => Ok(()),
                    PublicationCompanionRegistrationStop::OwnerUnavailable
                    | PublicationCompanionRegistrationStop::PublicationPending
                    | PublicationCompanionRegistrationStop::RebindRequired
                    | PublicationCompanionRegistrationStop::Superseded
                    | PublicationCompanionRegistrationStop::IdentityExhausted
                    | PublicationCompanionRegistrationStop::ForeignRuntime
                    | PublicationCompanionRegistrationStop::HeadUnavailable => Ok(()),
                };
            }
            Err(HeadCellRegistrationStop::LookupChanged) => return Ok(()),
        }
        let invalidation_owner = source_owner
            .invalidation_owner_ref()
            .as_any()
            .downcast_ref::<crate::domain_computation::primary_graph::SourceInvalidationOwner>()
            .expect("primary graph source carries SourceInvalidationOwner");
        let established = self.establish_restored(invalidation_owner);
        match established {
            Err(SettlementRegistrationStop::Admission(stop)) => Err(map_admission_stop(stop)),
            Ok(_)
            | Err(
                SettlementRegistrationStop::Alignment(_)
                | SettlementRegistrationStop::Foreign
                | SettlementRegistrationStop::SourceUnavailable
                | SettlementRegistrationStop::Edit(_),
            ) => Ok(()),
        }
    }
}

mod court_doors_for_test;

pub use pending_dependency::SelectedPendingConsumedOutput;
