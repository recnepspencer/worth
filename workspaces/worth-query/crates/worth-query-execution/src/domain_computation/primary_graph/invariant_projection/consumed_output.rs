use std::sync::Arc;
use std::sync::OnceLock;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;

use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;

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
pub(in crate::domain_computation::primary_graph) use verification::map_admission_stop;
pub(in crate::domain_computation::primary_graph) use verification::map_verification_stop;
pub(in crate::domain_computation::primary_graph) use verification::reserve_pending;
pub(in crate::domain_computation::primary_graph) use verification::ConsumedOutputVerification;
pub(in crate::domain_computation::primary_graph) use verification::ConsumedOutputVerificationStop;
pub(in crate::domain_computation::primary_graph) use verification::EvidenceView;
pub use verification::VerificationCapacityStop;

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
    /// The exact native basis on which the read sealed this current input.
    pub(in crate::domain_computation::primary_graph) fn current_read_basis(&self) -> (u64, u64) {
        (
            self.selected_native_root.runtime_instance_id(),
            self.selected_native_root.root_id(),
        )
    }

    pub(super) fn new(
        computation: crate::domain_computation::primary_graph::output_lineage::CurrentComputation,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: Arc<[ConsumedOutputEvidence]>,
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: Arc<OnceLock<SealedNativeOutputWitness>>,
        selected_native_root: Arc<PositionedRelationalSnapshot>,
    ) -> Self {
        Self {
            _computation: computation,
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness: Some(native_output_witness),
            selected_native_root,
        }
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
        )
    }

    /// Only the recovered root actually verified by a native reader may be
    /// retained without an executable source row. This is static settlement
    /// custody, never permission to reconstruct or refresh a source query.
    pub(in crate::domain_computation::primary_graph) fn claim_verified_checkpoint_root(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<bool, SettlementRegistrationStop> {
        if self.verification_requirement != Some(FullVerificationReason::CheckpointRestore)
            || !self.upstream.is_empty()
            || self
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get())
                .is_none()
            || !self.establish_restored(owner)?
        {
            return Ok(false);
        }
        Ok(matches!(
            owner.currentness(&self.selected_native_root, &self.identity)?,
            crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness::Clean
        ))
    }

    /// Record the restored root on the commit's meter. The read basis supplies
    /// only a branch identity to minting; Native selects its true head. A
    /// contended Native publication or changed lookup declines this derived
    /// registration, leaving the commit to compare the root in full again.
    pub(in crate::domain_computation::primary_graph) fn establish_restored_before_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_owner: &WorthQueryRelationalSourceOwner,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        if self.verification_requirement != Some(FullVerificationReason::CheckpointRestore) {
            return Ok(());
        }
        use crate::domain_computation::primary_graph::output_lineage::invalidation::HeadCellRegistrationStop;
        match source_owner.mint_mark_cell_at_head(execution, self.selected_native_root.branch_id())
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
        let established = self.establish_restored(&source_owner.invalidation_owner);
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
