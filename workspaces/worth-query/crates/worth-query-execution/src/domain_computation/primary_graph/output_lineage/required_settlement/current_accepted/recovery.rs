//! A committed member recovers derived registration without another publication.

use super::*;

impl AcceptedCurrentCandidate {
    pub(super) fn recover_registration<'basis>(
        &'basis self,
        computation: super::super::super::CurrentComputation,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        product: &'basis WorthQueryProductObservationLease,
        snapshot: &'basis SnapshotHandle,
        selected: &'basis PositionedRelationalSnapshot,
        facts: super::super::super::ComparableSourceFacts,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<CurrentAcceptedResult<'basis>, CurrentAcceptedStop> {
        let recorded = self.selected.recorded();
        let Some(witness) = self.selected.native_output_witness_cell() else {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        };
        let requirement = Some(FullVerificationReason::RegistrationIncomplete);
        let verified = match ConsumedOutputEvidence::verify_at_observation(
            &recorded.settlement_identity,
            &facts,
            &recorded.consumed_outputs,
            requirement,
            witness,
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        ) {
            Ok(answer) => answer,
            Err(ConsumedOutputVerificationStop::PendingUpstream) => {
                return self
                    .pending_consumed_output(owner, runtime, snapshot, selected, admission)
                    .map(|pending| match pending {
                        Some(pending) => CurrentAcceptedResult::PendingExact(pending),
                        None => CurrentAcceptedResult::PendingUnresolved,
                    })
                    .map_err(CurrentAcceptedStop::Closure);
            }
            Err(stop) => return Err(CurrentAcceptedStop::Closure(stop)),
        };
        if verified != ConsumedOutputVerification::Current {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        // Publication already owns the performed facts and sealed output.
        // Recover its derived row, retaining capacity stops on this caller.
        if recorded.consumed_outputs.is_empty() {
            if !owner
                .establish_verified_root(
                    selected,
                    &recorded.settlement_identity,
                    &facts,
                    witness.get().expect("selected performed witness is sealed"),
                    admission,
                )
                .map_err(CurrentAcceptedStop::Registration)?
            {
                return Ok(CurrentAcceptedResult::NeedsDisclosure);
            }
        } else {
            use crate::domain_computation::primary_graph::invariant_projection::PublicationRecoveryStop;
            match ConsumedOutputEvidence::recover_missing(
                owner,
                &recorded.consumed_outputs,
                runtime,
                snapshot,
                selected,
                admission,
            ) {
                Ok(()) => {}
                Err(PublicationRecoveryStop::Reason(_)) => {
                    return Ok(CurrentAcceptedResult::NeedsDisclosure)
                }
                Err(PublicationRecoveryStop::Closure(stop)) => {
                    return Err(CurrentAcceptedStop::Closure(stop))
                }
                Err(PublicationRecoveryStop::Registration(stop)) => {
                    return Err(CurrentAcceptedStop::Registration(stop))
                }
                Err(PublicationRecoveryStop::Admission(stop)) => {
                    return Err(CurrentAcceptedStop::Registration(
                        SettlementRegistrationStop::Admission(stop),
                    ))
                }
            }
            let upstream = super::super::super::invalidation::collect_consumed_output_upstream(
                &*recorded.consumed_outputs,
                admission,
            )
            .map_err(|stop| {
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(stop))
            })?;
            owner
                .establish_verified_consumed(
                    selected,
                    &recorded.settlement_identity,
                    &facts,
                    witness.get().expect("selected performed witness is sealed"),
                    upstream,
                    admission,
                )
                .map_err(CurrentAcceptedStop::Registration)?;
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
        let mut row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if row.verification_requirement != requirement || row.same_qualified_facts(&facts).is_none()
        {
            return Ok(CurrentAcceptedResult::NeedsDisclosure);
        }
        let cleanup = prepared.install().map_err(|stopped| {
            CurrentAcceptedStop::Registration(SettlementRegistrationStop::Edit(stopped.reason()))
        })?;
        row.verification_requirement = None;
        drop(row);
        Ok(CurrentAcceptedResult::Current(CurrentAcceptedOutput {
            _computation: computation,
            candidate: self,
            product,
            _snapshot: snapshot,
            _selected: selected,
            facts,
            _actor: cleanup,
        }))
    }
}
