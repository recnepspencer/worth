#[cfg(test)]
mod conflict_tests;
#[cfg(test)]
mod coordination_tests;
mod dependency;
#[cfg(test)]
mod evidence_tests;
mod family_proposal;
mod preflight;
mod proposal;
mod receipt;
mod reservation;
#[cfg(test)]
mod reveal_participation_tests;
#[cfg(feature = "certification-support")]
mod scale_certification;
mod settlement;
mod settlement_compiler;
#[cfg(test)]
mod shutdown_tests;
mod staged_reference;
mod staging;
mod terminal;
#[cfg(test)]
#[path = "unit_tests.rs"]
mod tests;

#[cfg(test)]
mod lifecycle {
    #[path = "tests.rs"]
    mod tests;
}

pub(in crate::runtime) use dependency::UiServiceProposalStage;
pub(in crate::runtime) use family_proposal::UiServiceFamilyProposal;
pub(in crate::runtime) use preflight::{
    UiPreflightedServiceProposal, UiServiceProposalPreflightDenial,
};
pub(in crate::runtime) use proposal::{
    UiServiceProposalCandidate, UiServiceProposalDemand, UiServiceProposalDemandConstructionDenial,
    UiServiceProposalIdentity,
};
#[cfg(test)]
pub(in crate::runtime) use receipt::{
    UiRecordedServiceProposalOwnerPort, UiRecordedServiceProposalPublicationPort,
};
pub(in crate::runtime) use receipt::{
    UiServiceProposalOwnerAcknowledgement, UiServiceProposalPublicationDisposition,
    UiServiceProposalPublicationReceipt, UiServiceProposalTerminalOwnerOutcome,
};
pub(in crate::runtime) use reservation::{
    UiReservedServiceProposal, UiServiceProposalBeforeEffectCancellationReceipt,
    UiServiceProposalReservationDenial, UiServiceProposalReservationOutcome,
};
#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::proposal_scale_evidence;
pub(in crate::runtime) use settlement::{
    UiServiceProposalPublicationDenial, UiServiceProposalSettlement,
    UiServiceProposalSettlementDenial,
};
pub(in crate::runtime) use staged_reference::{
    UiServiceMountedWorkReference, UiServiceProducedFactReference,
};
#[cfg(test)]
pub(in crate::runtime) use staging::UiServiceProposalStageIssuer;
pub(in crate::runtime) use staging::{
    UiServiceProposalStageReceipt, UiServiceProposalStagedBatch, UiServiceProposalStaging,
    UiServiceProposalStagingDenial,
};
pub(in crate::runtime) use terminal::{
    UiServiceProposalCompilerShutdownReceipt, UiServiceProposalTeardown,
    UiServiceProposalTeardownDenial, UiServiceProposalTerminalReason,
    UiServiceProposalTerminalReceipt,
};

#[derive(Debug)]
pub(in crate::runtime) struct UiServiceProposalCompiler {
    census: super::UiServiceProposalCensus,
    occupancy: super::occupancy::UiServiceProposalOccupancyTable,
    cancellations: super::cancellation::UiServiceProposalCancellationRegistry,
}

impl UiServiceProposalCompiler {
    pub(in crate::runtime) fn new() -> Self {
        Self {
            census: super::UiServiceProposalCensus::zero(),
            occupancy: super::occupancy::UiServiceProposalOccupancyTable::new(),
            cancellations: super::cancellation::UiServiceProposalCancellationRegistry::new(),
        }
    }

    pub(in crate::runtime) fn preflight(
        &mut self,
        candidate: UiServiceProposalCandidate,
        current: &super::UiServiceRequestCoherence,
        support: crate::capability::UiRuntimeServiceSupport,
    ) -> Result<UiPreflightedServiceProposal, UiServiceProposalPreflightDenial> {
        preflight::preflight(candidate, current, support)
    }

    pub(in crate::runtime) const fn census(&self) -> super::UiServiceProposalCensus {
        self.census
    }

    pub(in crate::runtime) fn reserve(
        &mut self,
        preflighted: UiPreflightedServiceProposal,
    ) -> Result<UiServiceProposalReservationOutcome, UiServiceProposalReservationDenial> {
        let plan = self
            .occupancy
            .plan(preflighted.candidate())
            .map_err(UiServiceProposalReservationDenial::Occupancy)?;
        if let Some(incumbent) = super::occupancy::UiServiceProposalOccupancyTable::coalesced(&plan)
        {
            return Ok(UiServiceProposalReservationOutcome::Coalesced { incumbent });
        }
        let displaced = plan.displacement();
        self.cancellations
            .can_reserve(
                preflighted.candidate().identity(),
                displaced.map(super::UiServiceProposalDisplacement::proposal),
            )
            .map_err(UiServiceProposalReservationDenial::Cancellation)?;
        let next_census = self
            .census
            .with_reservation(
                preflighted.candidate().family_proposals().len() as u16,
                displaced.map_or(0, super::UiServiceProposalDisplacement::released_leases),
                displaced.is_some(),
            )
            .map_err(UiServiceProposalReservationDenial::Census)?;
        let candidate = preflighted.into_candidate();
        let proposal = candidate.identity();
        let cancellation = candidate.cancellation();
        let (leases, displacement) = self.occupancy.commit(&candidate, plan);
        self.cancellations.reserve(
            proposal,
            cancellation,
            displacement.map(super::UiServiceProposalDisplacement::proposal),
        );
        self.census = next_census;
        Ok(UiServiceProposalReservationOutcome::Reserved(
            UiReservedServiceProposal::from_parts(candidate, leases, displacement),
        ))
    }

    pub(in crate::runtime) fn cancel_before_effect(
        &mut self,
        reservation: UiReservedServiceProposal,
    ) -> Result<UiServiceProposalBeforeEffectCancellationReceipt, UiServiceProposalReservationDenial>
    {
        let proposal = reservation.identity();
        self.occupancy
            .can_release(proposal, reservation.leases())
            .map_err(UiServiceProposalReservationDenial::Occupancy)?;
        self.cancellations
            .can_release(proposal, reservation.candidate().cancellation())
            .map_err(UiServiceProposalReservationDenial::Cancellation)?;
        let next_census = self
            .census
            .with_terminal_release(reservation.leases().len() as u16)
            .map_err(UiServiceProposalReservationDenial::Census)?;
        let released_leases = self.occupancy.release(proposal, reservation.leases());
        self.cancellations.release(proposal);
        self.census = next_census;
        Ok(UiServiceProposalBeforeEffectCancellationReceipt::new(
            proposal,
            released_leases,
        ))
    }

    /// Returns the reservation on denial so its occupancy lease, cancellation
    /// record, and census entries stay owned by a caller that can release them.
    pub(in crate::runtime) fn begin_staging(
        &mut self,
        reservation: UiReservedServiceProposal,
    ) -> Result<UiServiceProposalStaging, (UiReservedServiceProposal, UiServiceProposalStagingDenial)>
    {
        if let Err(denial) = self
            .occupancy
            .can_release(reservation.identity(), reservation.leases())
        {
            return Err((
                reservation,
                UiServiceProposalStagingDenial::Occupancy(denial),
            ));
        }
        Ok(UiServiceProposalStaging::new(reservation))
    }

    pub(in crate::runtime) fn advance_staging(
        &mut self,
        staging: &mut UiServiceProposalStaging,
        receipt: UiServiceProposalStageReceipt,
    ) -> Result<(), UiServiceProposalStagingDenial> {
        let closes_before_effect = staging.is_before_first_effect();
        if closes_before_effect {
            self.occupancy
                .can_release(staging.identity(), staging.leases())
                .map_err(UiServiceProposalStagingDenial::Occupancy)?;
        }
        let mut next_census = self.census;
        next_census
            .record_stage_receipt()
            .map_err(UiServiceProposalStagingDenial::Census)?;
        staging.accept_stage_receipt(receipt)?;
        if closes_before_effect {
            self.occupancy
                .close_before_effect_window(staging.identity(), staging.leases())
                .expect("first effect follows exact occupancy prevalidation");
        }
        self.census = next_census;
        Ok(())
    }

    pub(in crate::runtime) fn finish_staging(
        &self,
        staging: UiServiceProposalStaging,
    ) -> Result<
        UiServiceProposalStagedBatch,
        (UiServiceProposalStaging, UiServiceProposalStagingDenial),
    > {
        staging.finish()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn live_occupancy_count(&self) -> usize {
        self.occupancy.live_count()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn live_cancellation_count(&self) -> usize {
        self.cancellations.live_count()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn occupancy_work_counters(
        &self,
    ) -> super::UiServiceProposalOccupancyWorkCounters {
        self.occupancy.work_counters()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn live_neighborhood_count(&self) -> usize {
        self.occupancy.neighborhood_count()
    }
}
