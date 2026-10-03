//! Currentness of one exact accepted output, without repeating its source query.

use std::sync::Arc;

use worth_relational::facade::{
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    application_output_demand::ReadyCompletion,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
        SelectedPendingConsumedOutput,
    },
    SourceInvalidationOwner,
};

use super::{
    super::invalidation::{
        InvalidationEditAdmission, SettlementRegistrationStop, VerifiedCurrentCleanup,
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
    candidate: &'basis AcceptedCurrentCandidate,
    product: &'basis WorthQueryProductObservationLease,
    _snapshot: &'basis SnapshotHandle,
    _selected: &'basis PositionedRelationalSnapshot,
    facts: Arc<[WorthQueryApplicationObservedFact]>,
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
            && row
                .observed_source_facts
                .as_ref()
                .is_some_and(|facts| Arc::ptr_eq(facts, &self.facts));
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
            .ok_or_else(|| {
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
        if self.selected.native_output_witness().is_none() {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let facts = {
            let row = recorded
                .mutable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if row.verification_requirement.is_some() {
                return Ok(CurrentAcceptedResult::NeedsDisclosure);
            }
            let Some(facts) = row.observed_source_facts.as_ref() else {
                return Ok(CurrentAcceptedResult::NeedsDisclosure);
            };
            Arc::clone(facts)
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
                return ConsumedOutputEvidence::select_exact_pending_dependency(
                    &recorded.consumed_outputs,
                    owner,
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
                return ConsumedOutputEvidence::select_exact_pending_dependency(
                    &recorded.consumed_outputs,
                    owner,
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
        let row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if row.verification_requirement.is_some()
            || !row
                .observed_source_facts
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &facts))
        {
            drop(row);
            drop(prepared);
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let installed = prepared.install();
        drop(row);
        match installed {
            Ok(cleanup) => Ok(CurrentAcceptedResult::Current(CurrentAcceptedOutput {
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
