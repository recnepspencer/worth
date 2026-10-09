//! Currentness of one exact accepted output, without repeating its source query.

use worth_relational::facade::{
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    application_output_demand::ReadyCompletion,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
        SelectedPendingConsumedOutput,
    },
    SourceInvalidationOwner,
};

mod consumption;
#[cfg(test)]
mod own_write_recovery;
mod recovery;

use super::{
    super::invalidation::{
        FullVerificationReason, InvalidationEditAdmission, SettlementRegistrationStop,
        VerifiedCurrentCleanup,
    },
    AcceptedCurrentCandidate,
};

pub(in crate::domain_computation::primary_graph) enum CurrentAcceptedResult<'basis> {
    Current(CurrentAcceptedOutput<'basis>),
    PendingExact(SelectedPendingConsumedOutput<'basis>),
    PendingUnresolved,
    NeedsDisclosure,
}

/// The exact accepted row, admitted source basis, and actor image inspected as
/// one current output. Only the owner can mint it; the selected Query snapshot
/// keeps it inside the admitted read callback.
pub(in crate::domain_computation::primary_graph) struct CurrentAcceptedOutput<'basis> {
    _computation: super::super::CurrentComputation,
    candidate: &'basis AcceptedCurrentCandidate,
    product: &'basis WorthQueryProductObservationLease,
    _snapshot: &'basis SnapshotHandle,
    _selected: &'basis PositionedRelationalSnapshot,
    facts: super::super::ComparableSourceFacts,
    _actor: VerifiedCurrentCleanup,
}

/// A checked relation between the pinned exact completion and the selected
/// Product lease. Settlement construction consumes this value in the Query
/// admission callback, before the selected plan is released.
pub(in crate::domain_computation::primary_graph) struct BoundCurrentAcceptedOutput<'basis> {
    _proof: CurrentAcceptedOutput<'basis>,
}

impl<'basis> BoundCurrentAcceptedOutput<'basis> {
    pub(in crate::domain_computation::primary_graph) fn completion(&self) -> &ReadyCompletion {
        &self._proof.candidate.ready
    }

    pub(in crate::domain_computation::primary_graph) fn product(
        &self,
    ) -> &'basis WorthQueryProductObservationLease {
        self._proof.product
    }
}

impl<'basis> CurrentAcceptedOutput<'basis> {
    pub(in crate::domain_computation::primary_graph) fn bind_product(
        self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<BoundCurrentAcceptedOutput<'basis>>, CurrentAcceptedStop> {
        admission.charge_external_work(3).map_err(|stop| {
            CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(stop))
        })?;
        let recorded = self.candidate.selected.recorded();
        let row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let same_row = row.verification_requirement.is_none()
            && row.same_qualified_facts(&self.facts).is_some();
        drop(row);
        if !same_row {
            return Ok(None);
        }
        Ok(Some(BoundCurrentAcceptedOutput { _proof: self }))
    }
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CurrentAcceptedStop {
    Closure(ConsumedOutputVerificationStop),
    Registration(SettlementRegistrationStop),
}

impl AcceptedCurrentCandidate {
    pub(in crate::domain_computation::primary_graph) fn retain_recorded_identity(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        std::sync::Arc<super::super::RecordedSettlementIdentity>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        admission.charge_external_work(
            (std::mem::size_of::<std::sync::Arc<super::super::RecordedSettlementIdentity>>() + 1)
                as u64,
        )?;
        Ok(std::sync::Arc::clone(self.selected.settlement_identity()))
    }

    /// A changed Native correspondence cannot support this consumer's cutoff.
    /// Check it before retaining or re-verifying a successor that must run Fresh.
    pub(in crate::domain_computation::primary_graph) fn matches_consumed_output(
        &self,
        pending: &SelectedPendingConsumedOutput<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        match self.selected.native_output_witness() {
            Some(witness) => pending.matches_output_witness(witness, admission),
            None => Ok(false),
        }
    }

    /// A row verified in full executes again and reads the outputs it
    /// consumed. The first of them that changed refreshes before it does, so
    /// that execution reads every consumed output current.
    pub(in crate::domain_computation::primary_graph) fn pending_consumed_output<'selected>(
        &self,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'selected>>, ConsumedOutputVerificationStop>
    {
        if self.selected.recorded().consumed_outputs.is_empty() {
            return Ok(None);
        }
        if !self.own_evidence_is_current(runtime, snapshot, admission)? {
            return Ok(None);
        }
        ConsumedOutputEvidence::select_exact_pending_dependency(
            &self.selected.recorded().consumed_outputs,
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        )
    }

    fn own_evidence_is_current(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, ConsumedOutputVerificationStop> {
        // An old dependency set cannot constrain a consumer whose own native
        // evidence has changed. Missing evidence cannot reject the old edges.
        // This only requests disclosure, never Current.
        admission
            .charge_external_work(3)
            .map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?;
        let facts = self
            .selected
            .recorded()
            .observed_source_facts()
            .and_then(|facts| facts.for_comparison());
        let (Some(facts), Some(witness)) = (facts, self.selected.native_output_witness()) else {
            return Ok(true);
        };
        let Some(count) = self.selected.completed_handler_fact_count() else {
            return Ok(true);
        };
        let Some(handler_facts) = facts.get(..count) else {
            return Ok(true);
        };
        // Handler evidence decides whether the old edges still apply. Source
        // query stamps are a suffix whose meaning is checked by input equality.
        match ConsumedOutputEvidence::own_evidence_is_current(
            handler_facts,
            &self.selected.recorded().consumed_outputs,
            witness,
            runtime,
            snapshot,
            admission,
        ) {
            Ok(false) => Ok(false),
            Ok(true) | Err(ConsumedOutputVerificationStop::Unavailable) => Ok(true),
            Err(stop) => Err(stop),
        }
    }

    /// A complete actor posting set makes Clean a zero-fact-check proof. The
    /// recorded row and exact accepted cell remain pinned throughout the edit.
    pub(in crate::domain_computation::primary_graph) fn certify_current<'basis>(
        &'basis self,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        product: &'basis WorthQueryProductObservationLease,
        snapshot: &'basis SnapshotHandle,
        selected: &'basis PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<CurrentAcceptedResult<'basis>, CurrentAcceptedStop> {
        // Witness/row reads, the Arc pin, and the final guarded row comparison
        // are fixed selected visits. Reserve them before either row lock.
        admission.charge_external_work(10).map_err(|stop| {
            CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(stop))
        })?;
        let branch_work = u64::try_from(selected.branch_id().0.len())
            .ok()
            .and_then(|length| length.checked_add(3))
            .ok_or({
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                ))
            })?;
        admission
            .charge_external_work(branch_work)
            .map_err(|stop| {
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(stop))
            })?;
        let descriptor = product.relational_basis_descriptor();
        if descriptor.runtime_instance_id() != selected.runtime_instance_id()
            || descriptor.branch_id() != selected.branch_id()
            || descriptor.root_identity() != selected.root_id()
        {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let recorded = self.selected.recorded();
        let Some(qualified) = recorded
            .observed_source_facts()
            .and_then(|facts| facts.for_comparison())
        else {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        };
        let computation = qualified.computation();
        if self.selected.native_output_witness().is_none() {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let (requirement, facts) = {
            let row = recorded
                .mutable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (
                row.verification_requirement,
                row.same_qualified_facts(&qualified),
            )
        };
        // A restored output has a mark row only once it was compared in full
        // on this runtime. A clean row therefore discharges the restore
        // requirement below; without one this read needs source disclosure.
        if let (Some(FullVerificationReason::RegistrationIncomplete), Some(facts)) =
            (requirement, facts.as_ref())
        {
            if recorded.consumed_outputs.is_empty() {
                return self.recover_registration(
                    computation,
                    owner,
                    runtime,
                    product,
                    snapshot,
                    selected,
                    facts.clone(),
                    admission,
                );
            }
        }
        let restored = requirement == Some(FullVerificationReason::CheckpointRestore);
        if restored && facts.is_none() {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let (None, Some(facts)) = (requirement.filter(|_| !restored), facts) else {
            return match self.pending_consumed_output(owner, runtime, snapshot, selected, admission)
            {
                Ok(Some(pending)) => Ok(CurrentAcceptedResult::PendingExact(pending)),
                Ok(None) | Err(ConsumedOutputVerificationStop::Unavailable) => {
                    Ok(CurrentAcceptedResult::NeedsDisclosure)
                }
                Err(reason) => Err(CurrentAcceptedStop::Closure(reason)),
            };
        };
        match ConsumedOutputEvidence::verify_many_with_admission(
            &recorded.consumed_outputs,
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        ) {
            Ok(ConsumedOutputVerification::Current) => {}
            Ok(_) => {
                if !self
                    .own_evidence_is_current(runtime, snapshot, admission)
                    .map_err(CurrentAcceptedStop::Closure)?
                {
                    return Ok(CurrentAcceptedResult::NeedsDisclosure);
                }
                return ConsumedOutputEvidence::select_exact_pending_dependency(
                    &recorded.consumed_outputs,
                    owner,
                    runtime,
                    snapshot,
                    selected,
                    admission,
                )
                .map(|pending| match pending {
                    Some(pending) => CurrentAcceptedResult::PendingExact(pending),
                    None => CurrentAcceptedResult::NeedsDisclosure,
                })
                .map_err(CurrentAcceptedStop::Closure);
            }
            Err(ConsumedOutputVerificationStop::PendingUpstream) => {
                if !self
                    .own_evidence_is_current(runtime, snapshot, admission)
                    .map_err(CurrentAcceptedStop::Closure)?
                {
                    return Ok(CurrentAcceptedResult::NeedsDisclosure);
                }
                return ConsumedOutputEvidence::select_exact_pending_dependency(
                    &recorded.consumed_outputs,
                    owner,
                    runtime,
                    snapshot,
                    selected,
                    admission,
                )
                .map(|pending| match pending {
                    Some(pending) => CurrentAcceptedResult::PendingExact(pending),
                    None => CurrentAcceptedResult::PendingUnresolved,
                })
                .map_err(CurrentAcceptedStop::Closure);
            }
            Err(reason) => return Err(CurrentAcceptedStop::Closure(reason)),
        }
        let Some(prepared) = owner
            .prepare_verified_current(
                runtime,
                snapshot,
                selected,
                &recorded.settlement_identity,
                &facts,
                admission,
            )
            .map_err(CurrentAcceptedStop::Registration)?
        else {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        };
        // Restore or a local verification stop can change independently of the
        // actor image. The final check and same-image install share this guard.
        let mut row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if row.verification_requirement != requirement || row.same_qualified_facts(&facts).is_none()
        {
            drop(row);
            drop(prepared);
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let installed = prepared.install();
        if installed.is_ok() {
            row.verification_requirement = None;
        }
        drop(row);
        match installed {
            Ok(cleanup) => Ok(CurrentAcceptedResult::Current(CurrentAcceptedOutput {
                _computation: computation,
                candidate: self,
                product,
                _snapshot: snapshot,
                _selected: selected,
                facts,
                _actor: cleanup,
            })),
            Err(stopped) => {
                let reason = stopped.reason();
                drop(stopped);
                Err(CurrentAcceptedStop::Registration(
                    SettlementRegistrationStop::Edit(reason),
                ))
            }
        }
    }
}
