#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiServiceProposalTerminalReason {
    CancelledBeforePublication,
    RecoveryDisposed,
    AbandonedAtShutdown,
    PublicationAccepted,
    PublicationRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceProposalTerminalReceipt {
    proposal: super::UiServiceProposalIdentity,
    reason: UiServiceProposalTerminalReason,
    released_leases: u16,
    released_receipts: u16,
}

#[must_use]
#[derive(Debug)]
pub(in crate::runtime) struct UiServiceProposalTeardown {
    parts: super::staging::UiServiceProposalTerminalParts,
    reason: UiServiceProposalTerminalReason,
    acknowledged: super::super::UiServiceFamilyParticipation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceProposalCompilerShutdownReceipt {
    abandoned_proposals: u16,
    abandoned_leases: u16,
    final_census: super::super::UiServiceProposalCensus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiServiceProposalTeardownDenial {
    AwaitingOwnerSettlement(UiServiceProposalCompilerShutdownReceipt),
    Census(super::super::UiServiceProposalCensusDenial),
    Occupancy(super::super::UiServiceProposalOccupancyDenial),
    Cancellation(super::super::UiServiceProposalCancellationDenial),
    ForeignProposal,
    NonParticipatingFamily,
    OwnerScopeMismatch,
    ReasonMismatch,
    DuplicateOwnerOutcome,
    IncompleteOwnerDiscard,
}

impl UiServiceProposalTeardown {
    fn new(
        parts: super::staging::UiServiceProposalTerminalParts,
        reason: UiServiceProposalTerminalReason,
    ) -> Self {
        Self {
            parts,
            reason,
            acknowledged: super::super::UiServiceFamilyParticipation::EMPTY,
        }
    }

    pub(in crate::runtime) const fn proposal(&self) -> super::UiServiceProposalIdentity {
        self.parts.candidate.identity()
    }

    pub(in crate::runtime) fn is_complete(&self) -> bool {
        self.acknowledged.count() == self.parts.owners_requiring_discard.count()
    }

    fn accept(
        &mut self,
        outcome: super::UiServiceProposalTerminalOwnerOutcome,
    ) -> Result<(), UiServiceProposalTeardownDenial> {
        if outcome.proposal() != self.proposal() {
            return Err(UiServiceProposalTeardownDenial::ForeignProposal);
        }
        if outcome.reason() != self.reason {
            return Err(UiServiceProposalTeardownDenial::ReasonMismatch);
        }
        let family = outcome.family();
        if !self.parts.owners_requiring_discard.contains(family) {
            return Err(UiServiceProposalTeardownDenial::NonParticipatingFamily);
        }
        let proposal = self
            .parts
            .candidate
            .family_proposals()
            .iter()
            .find(|proposal| proposal.family() == family)
            .ok_or(UiServiceProposalTeardownDenial::NonParticipatingFamily)?;
        if proposal.scope() != outcome.scope() {
            return Err(UiServiceProposalTeardownDenial::OwnerScopeMismatch);
        }
        let next_acknowledged = self
            .acknowledged
            .with_family(family)
            .map_err(|_| UiServiceProposalTeardownDenial::DuplicateOwnerOutcome)?;
        let next_retained_receipts = self.parts.retained_receipts.checked_add(1).ok_or(
            UiServiceProposalTeardownDenial::Census(
                super::super::UiServiceProposalCensusDenial::Overflow,
            ),
        )?;
        self.acknowledged = next_acknowledged;
        self.parts.retained_receipts = next_retained_receipts;
        Ok(())
    }
}

impl super::UiServiceProposalCompiler {
    pub(in crate::runtime) fn shutdown_all_before_effect(
        &mut self,
    ) -> Result<UiServiceProposalCompilerShutdownReceipt, UiServiceProposalTeardownDenial> {
        let expected_proposals = self.occupancy.proposal_count();
        let expected_leases = u16::try_from(self.occupancy.live_count())
            .expect("occupancy holds at most OCCUPANCY_LIMIT leases");
        let expected_cancellations = u16::try_from(self.cancellations.live_count())
            .expect("cancellation holds at most CANCELLATION_RECORD_LIMIT records");
        if expected_proposals != expected_cancellations
            || !self.census.exactly_tracks_live(
                expected_proposals,
                expected_leases,
                expected_cancellations,
            )
        {
            return Err(UiServiceProposalTeardownDenial::Cancellation(
                super::super::UiServiceProposalCancellationDenial::ForeignProposal,
            ));
        }
        let (abandonable, abandoned_leases) = self.occupancy.before_effect_summary();
        if !self.cancellations.contains_all(&abandonable) {
            return Err(UiServiceProposalTeardownDenial::Cancellation(
                super::super::UiServiceProposalCancellationDenial::ForeignProposal,
            ));
        }
        let abandoned_proposals = u16::try_from(abandonable.len())
            .expect("cancellation holds at most CANCELLATION_RECORD_LIMIT records");
        let next_census = self
            .census
            .with_abandoned_before_effect(abandoned_proposals, abandoned_leases)
            .map_err(UiServiceProposalTeardownDenial::Census)?;
        let occupancy_released = self.occupancy.abandon_before_effect(&abandonable);
        let cancellations_released = self.cancellations.abandon(&abandonable);
        debug_assert_eq!(occupancy_released, abandoned_leases);
        debug_assert_eq!(cancellations_released, abandoned_proposals);
        self.census = next_census;
        let receipt = UiServiceProposalCompilerShutdownReceipt {
            abandoned_proposals,
            abandoned_leases,
            final_census: self.census,
        };
        if self.census.is_zero() {
            Ok(receipt)
        } else {
            Err(UiServiceProposalTeardownDenial::AwaitingOwnerSettlement(
                receipt,
            ))
        }
    }

    pub(in crate::runtime) fn cancel_staging(
        &mut self,
        staging: super::UiServiceProposalStaging,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            staging.into_terminal_parts(),
            UiServiceProposalTerminalReason::CancelledBeforePublication,
        )
    }

    pub(in crate::runtime) fn cancel_staged(
        &mut self,
        batch: super::UiServiceProposalStagedBatch,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            batch.into_terminal_parts(),
            UiServiceProposalTerminalReason::CancelledBeforePublication,
        )
    }

    #[cfg(test)]
    pub(in crate::runtime) fn shutdown_staging(
        &mut self,
        staging: super::UiServiceProposalStaging,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            staging.into_terminal_parts(),
            UiServiceProposalTerminalReason::AbandonedAtShutdown,
        )
    }

    pub(in crate::runtime) fn shutdown_staged(
        &mut self,
        batch: super::UiServiceProposalStagedBatch,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            batch.into_terminal_parts(),
            UiServiceProposalTerminalReason::AbandonedAtShutdown,
        )
    }

    pub(in crate::runtime) fn dispose_recovery_staged(
        &mut self,
        batch: super::UiServiceProposalStagedBatch,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            batch.into_terminal_parts(),
            UiServiceProposalTerminalReason::RecoveryDisposed,
        )
    }

    pub(in crate::runtime) fn acknowledge_terminal_owner(
        &mut self,
        teardown: &mut UiServiceProposalTeardown,
        outcome: super::UiServiceProposalTerminalOwnerOutcome,
    ) -> Result<(), UiServiceProposalTeardownDenial> {
        let mut next_census = self.census;
        next_census
            .record_stage_receipt()
            .map_err(UiServiceProposalTeardownDenial::Census)?;
        teardown.accept(outcome)?;
        self.census = next_census;
        Ok(())
    }

    pub(in crate::runtime) fn finish_teardown(
        &mut self,
        teardown: UiServiceProposalTeardown,
    ) -> Result<
        UiServiceProposalTerminalReceipt,
        (UiServiceProposalTeardown, UiServiceProposalTeardownDenial),
    > {
        if !teardown.is_complete() {
            return Err((
                teardown,
                UiServiceProposalTeardownDenial::IncompleteOwnerDiscard,
            ));
        }
        match self.release_terminal_parts(teardown.parts, teardown.reason) {
            Ok(receipt) => Ok(receipt),
            Err(denial) => Err((
                UiServiceProposalTeardown {
                    parts: denial.0,
                    reason: teardown.reason,
                    acknowledged: teardown.acknowledged,
                },
                denial.1,
            )),
        }
    }

    pub(in crate::runtime) fn shutdown_reservation(
        &mut self,
        reservation: super::UiReservedServiceProposal,
    ) -> Result<
        UiServiceProposalTerminalReceipt,
        (
            super::UiReservedServiceProposal,
            UiServiceProposalTeardownDenial,
        ),
    > {
        let (candidate, leases, displacement) = reservation.into_parts();
        let parts = super::staging::UiServiceProposalTerminalParts {
            candidate,
            leases,
            retained_receipts: 0,
            owners_requiring_discard: super::super::UiServiceFamilyParticipation::EMPTY,
        };
        self.release_terminal_parts(parts, UiServiceProposalTerminalReason::AbandonedAtShutdown)
            .map_err(|(parts, denial)| {
                (
                    super::UiReservedServiceProposal::from_parts(
                        *parts.candidate,
                        parts.leases,
                        displacement,
                    ),
                    denial,
                )
            })
    }

    #[cfg(test)]
    pub(in crate::runtime) fn shutdown_awaiting_settlement(
        &mut self,
        settlement: super::UiServiceProposalSettlement,
    ) -> UiServiceProposalTeardown {
        UiServiceProposalTeardown::new(
            settlement.into_shutdown_terminal_parts(),
            UiServiceProposalTerminalReason::AbandonedAtShutdown,
        )
    }

    fn release_terminal_parts(
        &mut self,
        parts: super::staging::UiServiceProposalTerminalParts,
        reason: UiServiceProposalTerminalReason,
    ) -> Result<
        UiServiceProposalTerminalReceipt,
        (
            super::staging::UiServiceProposalTerminalParts,
            UiServiceProposalTeardownDenial,
        ),
    > {
        let proposal = parts.candidate.identity();
        if let Err(denial) = self.occupancy.can_release(proposal, &parts.leases) {
            return Err((parts, UiServiceProposalTeardownDenial::Occupancy(denial)));
        }
        if let Err(denial) = self
            .cancellations
            .can_release(proposal, parts.candidate.cancellation())
        {
            return Err((parts, UiServiceProposalTeardownDenial::Cancellation(denial)));
        }
        let next_census = match self.census.with_complete_release(
            u16::try_from(parts.leases.len())
                .expect("occupancy holds at most OCCUPANCY_LIMIT leases"),
            parts.retained_receipts,
        ) {
            Ok(census) => census,
            Err(denial) => return Err((parts, UiServiceProposalTeardownDenial::Census(denial))),
        };
        let released_leases = self.occupancy.release(proposal, &parts.leases);
        self.cancellations.release(proposal);
        self.census = next_census;
        Ok(UiServiceProposalTerminalReceipt::new(
            proposal,
            reason,
            released_leases,
            parts.retained_receipts,
        ))
    }
}

impl UiServiceProposalTerminalReceipt {
    pub(super) const fn new(
        proposal: super::UiServiceProposalIdentity,
        reason: UiServiceProposalTerminalReason,
        released_leases: u16,
        released_receipts: u16,
    ) -> Self {
        Self {
            proposal,
            reason,
            released_leases,
            released_receipts,
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn reason(self) -> UiServiceProposalTerminalReason {
        self.reason
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn released_leases(self) -> u16 {
        self.released_leases
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn released_receipts(self) -> u16 {
        self.released_receipts
    }
}

impl UiServiceProposalCompilerShutdownReceipt {
    #[cfg(test)]
    pub(in crate::runtime) const fn abandoned_proposals(self) -> u16 {
        self.abandoned_proposals
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn abandoned_leases(self) -> u16 {
        self.abandoned_leases
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn final_census(self) -> super::super::UiServiceProposalCensus {
        self.final_census
    }

    pub(in crate::runtime) const fn is_complete(self) -> bool {
        self.final_census.is_zero()
    }
}
