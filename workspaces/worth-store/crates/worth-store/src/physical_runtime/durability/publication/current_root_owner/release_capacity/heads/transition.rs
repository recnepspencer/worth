//! Local validation under the roster owner's lock. Full canonical commitment
//! belongs to checkpoint/rejoin, not each selected single-key transition.

use super::*;

pub(in super::super) struct PreparedHeadTransition {
    step: SelectedReleaseHeadStep,
    slot: Result<usize, usize>,
}

impl SelectedReleaseHeadStep {
    pub(in super::super) fn prepare(
        self,
        roster: &SelectedReleaseHeadRoster,
    ) -> Result<PreparedHeadTransition, ReleaseCertificateCapacityDenial> {
        let denial = ReleaseCertificateCapacityDenial::SelectedFactMismatch;
        if roster.root != self.source_root {
            return Err(denial);
        }
        let slot = roster
            .entries
            .binary_search_by_key(&self.mutation.key(), |entry| entry.key());
        if slot.ok().map(|index| roster.entries[index]) != self.mutation.expected_prior() {
            return Err(denial);
        }
        match self.mutation {
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior,
                next,
            } if valid_successor(expected_prior, next) => {}
            ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior }
                if expected_prior.terminal() && slot.is_ok() => {}
            _ => return Err(denial),
        }
        let transition = PreparedHeadTransition { step: self, slot };
        let (first, last) = transition.projected_endpoints(roster);
        if self.result_root.is_some() != first.is_some()
            || self
                .result_root
                .is_some_and(|root| Some(root.first()) != first || Some(root.last()) != last)
        {
            return Err(denial);
        }
        Ok(transition)
    }
}

impl PreparedHeadTransition {
    pub(super) fn inserts(&self) -> bool {
        matches!(
            self.step.mutation,
            ReleaseCustodyHeadMutationV1::Upsert { .. }
        ) && self.slot.is_err()
    }

    pub(in super::super) fn has_backing(&self, roster: &SelectedReleaseHeadRoster) -> bool {
        !self.inserts() || roster.entries.len() < roster.entries.capacity()
    }

    /// Caller holds the same owner lock throughout prepare and apply. All
    /// fallible joins and mandatory backing checks precede this mutation.
    pub(in super::super) fn apply(self, roster: &mut SelectedReleaseHeadRoster) {
        match self.step.mutation {
            ReleaseCustodyHeadMutationV1::Upsert { next, .. } => match self.slot {
                Ok(index) => roster.entries[index] = next,
                Err(index) => roster.entries.insert(index, next),
            },
            ReleaseCustodyHeadMutationV1::RetireTerminal { .. } => {
                roster
                    .entries
                    .remove(self.slot.expect("validated terminal membership"));
            }
        }
        roster.root = self.step.result_root;
    }

    fn projected_endpoints(
        &self,
        roster: &SelectedReleaseHeadRoster,
    ) -> (
        Option<ReleaseCustodyHeadKeyV1>,
        Option<ReleaseCustodyHeadKeyV1>,
    ) {
        let entries = &roster.entries;
        let first = entries.first().map(|entry| entry.key());
        let last = entries.last().map(|entry| entry.key());
        match self.step.mutation {
            ReleaseCustodyHeadMutationV1::Upsert { next, .. } => (
                Some(first.map_or(next.key(), |key| key.min(next.key()))),
                Some(last.map_or(next.key(), |key| key.max(next.key()))),
            ),
            ReleaseCustodyHeadMutationV1::RetireTerminal { .. } => {
                let index = self.slot.expect("validated terminal membership");
                (
                    if index == 0 {
                        entries.get(1).map(|entry| entry.key())
                    } else {
                        first
                    },
                    if index + 1 == entries.len() {
                        entries.len().checked_sub(2).map(|last| entries[last].key())
                    } else {
                        last
                    },
                )
            }
        }
    }
}
