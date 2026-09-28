const CANCELLATION_RECORD_LIMIT: usize = 64;

#[derive(Debug)]
pub(super) struct UiServiceProposalCancellationRegistry {
    records: Vec<UiServiceProposalCancellationRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiServiceProposalCancellationRecord {
    proposal: super::UiServiceProposalIdentity,
    cancellation: super::UiServiceCancellationIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiServiceProposalCancellationDenial {
    CapacityExceeded,
    DuplicateProposal,
    ForeignProposal,
}

impl UiServiceProposalCancellationRegistry {
    pub(super) const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub(super) fn can_reserve(
        &self,
        proposal: super::UiServiceProposalIdentity,
        displaced: Option<super::UiServiceProposalIdentity>,
    ) -> Result<(), UiServiceProposalCancellationDenial> {
        if self
            .records
            .iter()
            .any(|record| record.proposal == proposal)
        {
            return Err(UiServiceProposalCancellationDenial::DuplicateProposal);
        }
        let displaced_count = usize::from(displaced.is_some_and(|candidate| {
            self.records
                .iter()
                .any(|record| record.proposal == candidate)
        }));
        if self.records.len() - displaced_count + 1 > CANCELLATION_RECORD_LIMIT {
            return Err(UiServiceProposalCancellationDenial::CapacityExceeded);
        }
        Ok(())
    }

    pub(super) fn reserve(
        &mut self,
        proposal: super::UiServiceProposalIdentity,
        cancellation: super::UiServiceCancellationIdentity,
        displaced: Option<super::UiServiceProposalIdentity>,
    ) {
        if let Some(displaced) = displaced {
            self.records.retain(|record| record.proposal != displaced);
        }
        self.records.push(UiServiceProposalCancellationRecord {
            proposal,
            cancellation,
        });
    }

    pub(super) fn can_release(
        &self,
        proposal: super::UiServiceProposalIdentity,
        cancellation: super::UiServiceCancellationIdentity,
    ) -> Result<(), UiServiceProposalCancellationDenial> {
        if !self
            .records
            .iter()
            .any(|record| record.proposal == proposal && record.cancellation == cancellation)
        {
            return Err(UiServiceProposalCancellationDenial::ForeignProposal);
        }
        Ok(())
    }

    pub(super) fn release(&mut self, proposal: super::UiServiceProposalIdentity) {
        self.records.retain(|record| record.proposal != proposal);
    }

    pub(super) fn live_count(&self) -> usize {
        self.records.len()
    }

    pub(super) fn contains_all(&self, proposals: &[super::UiServiceProposalIdentity]) -> bool {
        proposals.iter().all(|proposal| {
            self.records
                .iter()
                .any(|record| record.proposal == *proposal)
        })
    }

    pub(super) fn abandon(&mut self, proposals: &[super::UiServiceProposalIdentity]) -> u16 {
        let before = self.records.len();
        self.records
            .retain(|record| !proposals.contains(&record.proposal));
        u16::try_from(before - self.records.len())
            .expect("cancellation holds at most CANCELLATION_RECORD_LIMIT records")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiServiceProposalCancellationDenial, UiServiceProposalCancellationRegistry,
        CANCELLATION_RECORD_LIMIT,
    };

    #[test]
    fn cancellation_capacity_denial_preserves_all_incumbents() {
        let mut registry = UiServiceProposalCancellationRegistry::new();
        for index in 0..CANCELLATION_RECORD_LIMIT {
            let value = index as u64 + 1;
            let proposal = super::super::UiServiceProposalIdentity::for_test(value);
            registry.can_reserve(proposal, None).unwrap();
            registry.reserve(
                proposal,
                super::super::UiServiceCancellationIdentity::for_test(value),
                None,
            );
        }
        let overflow = super::super::UiServiceProposalIdentity::for_test(100);
        assert_eq!(
            registry.can_reserve(overflow, None),
            Err(UiServiceProposalCancellationDenial::CapacityExceeded)
        );
        assert_eq!(registry.live_count(), CANCELLATION_RECORD_LIMIT);
    }
}
