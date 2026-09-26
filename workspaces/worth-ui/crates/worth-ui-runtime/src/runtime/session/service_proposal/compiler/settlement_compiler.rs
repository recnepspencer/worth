impl super::UiServiceProposalCompiler {
    pub(in crate::runtime) fn begin_settlement(
        &mut self,
        batch: super::UiServiceProposalStagedBatch,
        publication: super::UiServiceProposalPublicationReceipt,
    ) -> Result<
        super::UiServiceProposalSettlement,
        (
            super::UiServiceProposalStagedBatch,
            super::UiServiceProposalPublicationDenial,
        ),
    > {
        let retained_receipts =
            match super::UiServiceProposalSettlement::validate_publication(&batch, publication) {
                Ok(count) => count,
                Err(denial) => return Err((batch, denial)),
            };
        let mut next_census = self.census;
        if let Err(denial) = next_census.record_stage_receipt() {
            return Err((
                batch,
                super::UiServiceProposalPublicationDenial::Census(denial),
            ));
        }
        let settlement = super::UiServiceProposalSettlement::from_validated_publication(
            batch,
            publication,
            retained_receipts,
        );
        self.census = next_census;
        Ok(settlement)
    }

    pub(in crate::runtime) fn acknowledge_owner(
        &mut self,
        settlement: &mut super::UiServiceProposalSettlement,
        acknowledgement: super::UiServiceProposalOwnerAcknowledgement,
    ) -> Result<(), super::UiServiceProposalSettlementDenial> {
        let mut next_census = self.census;
        next_census
            .record_stage_receipt()
            .map_err(super::UiServiceProposalSettlementDenial::Census)?;
        settlement.accept_owner_acknowledgement(acknowledgement)?;
        self.census = next_census;
        Ok(())
    }

    pub(in crate::runtime) fn finish_settlement(
        &mut self,
        settlement: super::UiServiceProposalSettlement,
    ) -> Result<
        super::UiServiceProposalTerminalReceipt,
        (
            super::UiServiceProposalSettlement,
            super::UiServiceProposalSettlementDenial,
        ),
    > {
        if !settlement.is_complete() {
            return Err((
                settlement,
                super::UiServiceProposalSettlementDenial::IncompleteOwnerSettlement,
            ));
        }
        let proposal = settlement.candidate().identity();
        let cancellation = settlement.candidate().cancellation();
        if let Err(denial) = self.occupancy.can_release(proposal, settlement.leases()) {
            return Err((
                settlement,
                super::UiServiceProposalSettlementDenial::Occupancy(denial),
            ));
        }
        if let Err(denial) = self.cancellations.can_release(proposal, cancellation) {
            return Err((
                settlement,
                super::UiServiceProposalSettlementDenial::Cancellation(denial),
            ));
        }
        let disposition = settlement.publication().disposition();
        let lease_count = u16::try_from(settlement.leases().len())
            .expect("occupancy holds at most OCCUPANCY_LIMIT leases");
        let receipt_count = settlement.retained_receipt_count();
        let next_census = match self
            .census
            .with_complete_release(lease_count, receipt_count)
        {
            Ok(census) => census,
            Err(denial) => {
                return Err((
                    settlement,
                    super::UiServiceProposalSettlementDenial::Census(denial),
                ));
            }
        };
        let parts = settlement
            .terminal_parts()
            .expect("complete settlement was validated before terminal extraction");
        let released_leases = self.occupancy.release(proposal, &parts.leases);
        self.cancellations.release(proposal);
        self.census = next_census;
        let reason = match disposition {
            super::UiServiceProposalPublicationDisposition::Accepted => {
                super::UiServiceProposalTerminalReason::PublicationAccepted
            }
            super::UiServiceProposalPublicationDisposition::Rejected => {
                super::UiServiceProposalTerminalReason::PublicationRejected
            }
        };
        Ok(super::UiServiceProposalTerminalReceipt::new(
            proposal,
            reason,
            released_leases,
            parts.retained_receipts,
        ))
    }
}
