//! Allocation-free comparison target for the exact pre-pending head roster.
//! Historical authority comes from the independent ordered C9/media walk.

use worth_store_physical_format::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadRosterDigestV1,
};

use super::Denial;

pub(super) struct InversePendingUpsert<'a> {
    final_entries: &'a [ReleaseCustodyHeadEntryV1],
    pending_index: usize,
    prior: Option<ReleaseCustodyHeadEntryV1>,
    count: u64,
}

impl<'a> InversePendingUpsert<'a> {
    pub(super) fn new(
        final_entries: &'a [ReleaseCustodyHeadEntryV1],
        mutation: ReleaseCustodyHeadMutationV1,
    ) -> Result<Self, Denial> {
        let ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior,
            next,
        } = mutation
        else {
            return Err(Denial::CertificateRoster);
        };
        let pending_index = final_entries
            .binary_search_by_key(&next.key(), |entry| entry.key())
            .map_err(|_| Denial::CertificateRoster)?;
        if final_entries[pending_index] != next
            || expected_prior.is_some_and(|prior| prior.key() != next.key())
        {
            return Err(Denial::CertificateRoster);
        }
        let final_count = u64::try_from(final_entries.len()).map_err(|_| Denial::BoundExceeded)?;
        let count = if expected_prior.is_some() {
            final_count
        } else {
            final_count
                .checked_sub(1)
                .ok_or(Denial::CertificateRoster)?
        };
        Ok(Self {
            final_entries,
            pending_index,
            prior: expected_prior,
            count,
        })
    }

    pub(super) fn count(&self) -> u64 {
        self.count
    }

    fn entries(&self) -> impl Iterator<Item = ReleaseCustodyHeadEntryV1> + '_ {
        self.final_entries
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, entry)| {
                if index == self.pending_index {
                    self.prior
                } else {
                    Some(entry)
                }
            })
    }

    pub(super) fn digest(
        &self,
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    ) -> Result<(u64, [u8; 32]), Denial> {
        let mut digest = ReleaseCustodyHeadRosterDigestV1::new(source_root, self.count);
        for entry in self.entries() {
            digest.push(entry).map_err(|_| Denial::CertificateRoster)?;
        }
        Ok(digest.finish())
    }

    pub(super) fn matches(&self, observed: &[ReleaseCustodyHeadEntryV1]) -> bool {
        observed.iter().copied().eq(self.entries())
    }
}
