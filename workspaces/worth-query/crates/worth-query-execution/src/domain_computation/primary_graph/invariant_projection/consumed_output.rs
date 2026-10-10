use std::mem::{align_of, size_of};
use std::sync::{Arc, OnceLock};

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::{
            FullVerificationReason, InvalidationEditAdmission, RetainedConsumedOutputCapacity,
            SettlementRegistrationStop,
        },
        RecordedSettlementIdentity, SealedNativeOutputWitness,
    },
    SourceInvalidationOwner,
};

#[cfg(feature = "certification-invalidation-equivalence")]
mod equivalence;
mod pending_dependency;
mod publication_recovery;
pub(in crate::domain_computation::primary_graph) use publication_recovery::PublicationRecoveryStop;
mod rebound;
mod requested_read;
#[cfg(test)]
mod test_support;
mod verification;
pub(in crate::domain_computation::primary_graph) use pending_dependency::SelectedPendingConsumedOutput;
pub(in crate::domain_computation::primary_graph::invariant_projection) use verification::map_admission_stop;
#[cfg(feature = "certification-invalidation-equivalence")]
pub(in crate::domain_computation::primary_graph) use verification::EvidenceView;
pub(in crate::domain_computation::primary_graph) use verification::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop,
};

/// One output actually selected by a projected read. The lineage owner minted
/// the identity; Arc only shares retained evidence and is not an identity.
/// This runtime-only record is never serialized through a TypeId-bearing key.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ConsumedOutputEvidence {
    _computation: crate::domain_computation::primary_graph::output_lineage::CurrentComputation,
    identity: Arc<RecordedSettlementIdentity>,
    source_facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
    upstream: Arc<[ConsumedOutputEvidence]>,
    verification_requirement: Option<FullVerificationReason>,
    native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
    selected_native_root: Arc<PositionedRelationalSnapshot>,
    _capacity: RetainedConsumedOutputCapacity,
    backing_capacity: Option<Arc<RetainedInvalidationCapacity>>,
}

impl PartialEq for ConsumedOutputEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
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

impl Eq for ConsumedOutputEvidence {}

impl ConsumedOutputEvidence {
    pub(super) fn new(
        computation: crate::domain_computation::primary_graph::output_lineage::CurrentComputation,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: Arc<[ConsumedOutputEvidence]>,
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: Arc<OnceLock<SealedNativeOutputWitness>>,
        selected_native_root: Arc<PositionedRelationalSnapshot>,
        capacity: RetainedConsumedOutputCapacity,
    ) -> Self {
        Self {
            _computation: computation,
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness: Some(native_output_witness),
            selected_native_root,
            _capacity: capacity,
            backing_capacity: None,
        }
    }

    pub(super) const fn metadata_bytes() -> u64 {
        (size_of::<Self>()
            + size_of::<RecordedSettlementIdentity>()
            + size_of::<usize>() * 2
            + align_of::<RecordedSettlementIdentity>() * 2) as u64
    }

    /// Admits the one shared allocation `consumed` is held in, on the meter
    /// of the request that consumed them. Every edge keeps the ticket, so a
    /// selected upstream edge keeps the backing funded after the lineage and
    /// attempt that consumed it retire.
    pub(in crate::domain_computation::primary_graph) fn admit_backing(
        consumed: &mut [Self],
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        if consumed.is_empty() {
            return Ok(());
        }
        let bytes = consumed
            .len()
            .checked_mul(size_of::<Self>())
            .and_then(|bytes| bytes.checked_add(size_of::<usize>() * 2 + align_of::<Self>() * 2))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
        let backing = owner
            .retain_consumed_output_backing(bytes, admission)
            .map_err(map_admission_stop)?;
        for edge in consumed {
            edge.backing_capacity = Some(Arc::clone(&backing));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }

    /// A consumed restored output gets its mark row at the basis this reader
    /// compared it in full; any other output answers for its own row. Returns
    /// whether a consumer may register over this output.
    pub(in crate::domain_computation::primary_graph) fn establish_restored(
        &self,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, SettlementRegistrationStop> {
        if self.verification_requirement != Some(FullVerificationReason::CheckpointRestore) {
            return Ok(true);
        }
        let witness = self.native_output_witness.as_ref();
        let Some(witness) = witness.and_then(|witness| witness.get()) else {
            return Ok(false);
        };
        if !self.upstream.is_empty() {
            return Ok(false);
        }
        owner.establish_verified_root(
            &self.selected_native_root,
            &self.identity,
            &self.source_facts,
            witness,
            admission,
        )
    }

    /// Only the recovered root actually verified by a native reader may be
    /// retained without an executable source row. This is static settlement
    /// custody, never permission to reconstruct or refresh a source query.
    pub(in crate::domain_computation::primary_graph) fn claim_verified_checkpoint_root(
        &self,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, SettlementRegistrationStop> {
        if self.verification_requirement != Some(FullVerificationReason::CheckpointRestore)
            || !self.upstream.is_empty()
            || self
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get())
                .is_none()
            || !self.establish_restored(owner, admission)?
        {
            return Ok(false);
        }
        Ok(matches!(
            owner.currentness(&self.selected_native_root, &self.identity, admission)?,
            crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness::Clean
        ))
    }

    /// Record the restored root on the commit's meter. The read basis supplies
    /// only a branch identity to minting; Native selects its true head. A
    /// contended Native publication or changed lookup declines this derived
    /// registration, leaving the commit to compare the root in full again.
    pub(in crate::domain_computation::primary_graph) fn establish_restored_before_commit(
        &self,
        source_owner: &WorthQueryRelationalSourceOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        if self.verification_requirement != Some(FullVerificationReason::CheckpointRestore) {
            return Ok(());
        }
        use crate::domain_computation::primary_graph::output_lineage::invalidation::HeadCellRegistrationStop;
        match source_owner.mint_mark_cell_at_head(self.selected_native_root.branch_id(), admission)
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
        let established = self.establish_restored(&source_owner.invalidation_owner, admission);
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
