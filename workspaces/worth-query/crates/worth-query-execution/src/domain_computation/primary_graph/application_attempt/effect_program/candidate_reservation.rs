use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidatePolicy,
    ApplicationCandidateRequirements,
};

use super::super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

#[derive(Clone, Copy)]
pub(super) enum CandidateItemKind {
    Create,
    Delete,
    Link,
    Unlink,
    Write,
    Emit,
}

pub(super) struct WorthQueryCandidateReservation {
    requested: ApplicationCandidateRequirements,
    remaining: ApplicationCandidateCardinalityCeiling,
    remaining_items: usize,
    remaining_retained_representation_bytes: usize,
}

impl WorthQueryCandidateReservation {
    pub(super) fn admit(
        requested: ApplicationCandidateRequirements,
        policy: ApplicationCandidatePolicy,
        candidate_item_capacity: Option<u64>,
        retained_representation_byte_capacity: Option<u64>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let requested_cardinality = requested.cardinality();
        let requested_total = Self::check_host_capacity(
            requested,
            candidate_item_capacity,
            retained_representation_byte_capacity,
        )?;
        let within_binding = policy.cardinality().is_none_or(|ceiling| {
            requested_cardinality.maximum_creates() <= ceiling.maximum_creates()
                && requested_cardinality.maximum_deletes() <= ceiling.maximum_deletes()
                && requested_cardinality.maximum_links() <= ceiling.maximum_links()
                && requested_cardinality.maximum_unlinks() <= ceiling.maximum_unlinks()
                && requested_cardinality.maximum_writes() <= ceiling.maximum_writes()
                && requested_cardinality.maximum_emits() <= ceiling.maximum_emits()
        }) && policy.resources().is_none_or(|ceiling| {
            requested
                .resources()
                .maximum_retained_representation_bytes()
                <= ceiling.maximum_retained_representation_bytes()
        });
        if !within_binding {
            return Err(capacity_denial());
        }
        Ok(Self {
            requested,
            remaining: requested_cardinality,
            remaining_items: requested_total,
            remaining_retained_representation_bytes: requested
                .resources()
                .maximum_retained_representation_bytes(),
        })
    }

    pub(super) fn requirements(&self) -> ApplicationCandidateRequirements {
        self.requested
    }

    pub(super) fn check_host_capacity(
        requested: ApplicationCandidateRequirements,
        items: Option<u64>,
        bytes: Option<u64>,
    ) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        let count = total(requested.cardinality()).ok_or_else(capacity_denial)?;
        let fits = u64::try_from(count)
            .is_ok_and(|count| items.is_none_or(|maximum| count <= maximum))
            && u64::try_from(
                requested
                    .resources()
                    .maximum_retained_representation_bytes(),
            )
            .is_ok_and(|bytes_requested| bytes.is_none_or(|maximum| bytes_requested <= maximum));
        if !fits {
            return Err(capacity_denial());
        }
        Ok(count)
    }

    /// Reserves a write and the remaining representation after ordinary effects.
    pub(super) fn reserve_deferred_initializer(
        &mut self,
    ) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        let bytes = self.remaining_retained_representation_bytes;
        self.charge_retained_representation(CandidateItemKind::Write, bytes, 0)?;
        Ok(bytes)
    }

    pub(super) fn total_items(&self) -> usize {
        self.remaining_items
    }

    pub(super) fn charge(
        &mut self,
        kind: CandidateItemKind,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.charge_many(kind, 1, 0, 0)
    }

    pub(super) fn charge_retained_representation(
        &mut self,
        kind: CandidateItemKind,
        retained_representation_bytes: usize,
        replaced_representation_bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.charge_many(
            kind,
            1,
            retained_representation_bytes,
            replaced_representation_bytes,
        )
    }

    pub(super) fn charge_many(
        &mut self,
        kind: CandidateItemKind,
        count: usize,
        retained_representation_bytes: usize,
        replaced_representation_bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let remaining_items = self
            .remaining_items
            .checked_sub(count)
            .ok_or_else(reservation_denial)?;
        let remaining = match kind {
            CandidateItemKind::Create => self.remaining.maximum_creates(),
            CandidateItemKind::Delete => self.remaining.maximum_deletes(),
            CandidateItemKind::Link => self.remaining.maximum_links(),
            CandidateItemKind::Unlink => self.remaining.maximum_unlinks(),
            CandidateItemKind::Write => self.remaining.maximum_writes(),
            CandidateItemKind::Emit => self.remaining.maximum_emits(),
        };
        let Some(remaining) = remaining.checked_sub(count) else {
            return Err(reservation_denial());
        };
        let next_cardinality = match kind {
            CandidateItemKind::Create => ApplicationCandidateCardinalityCeiling::fixed(
                remaining,
                self.remaining.maximum_deletes(),
                self.remaining.maximum_links(),
                self.remaining.maximum_unlinks(),
                self.remaining.maximum_writes(),
                self.remaining.maximum_emits(),
            ),
            CandidateItemKind::Delete => ApplicationCandidateCardinalityCeiling::fixed(
                self.remaining.maximum_creates(),
                remaining,
                self.remaining.maximum_links(),
                self.remaining.maximum_unlinks(),
                self.remaining.maximum_writes(),
                self.remaining.maximum_emits(),
            ),
            CandidateItemKind::Link => ApplicationCandidateCardinalityCeiling::fixed(
                self.remaining.maximum_creates(),
                self.remaining.maximum_deletes(),
                remaining,
                self.remaining.maximum_unlinks(),
                self.remaining.maximum_writes(),
                self.remaining.maximum_emits(),
            ),
            CandidateItemKind::Unlink => ApplicationCandidateCardinalityCeiling::fixed(
                self.remaining.maximum_creates(),
                self.remaining.maximum_deletes(),
                self.remaining.maximum_links(),
                remaining,
                self.remaining.maximum_writes(),
                self.remaining.maximum_emits(),
            ),
            CandidateItemKind::Write => ApplicationCandidateCardinalityCeiling::fixed(
                self.remaining.maximum_creates(),
                self.remaining.maximum_deletes(),
                self.remaining.maximum_links(),
                self.remaining.maximum_unlinks(),
                remaining,
                self.remaining.maximum_emits(),
            ),
            CandidateItemKind::Emit => ApplicationCandidateCardinalityCeiling::fixed(
                self.remaining.maximum_creates(),
                self.remaining.maximum_deletes(),
                self.remaining.maximum_links(),
                self.remaining.maximum_unlinks(),
                self.remaining.maximum_writes(),
                remaining,
            ),
        };
        let available_bytes = self
            .remaining_retained_representation_bytes
            .checked_add(replaced_representation_bytes)
            .ok_or_else(reservation_denial)?;
        let remaining_retained_representation_bytes = available_bytes
            .checked_sub(retained_representation_bytes)
            .ok_or_else(reservation_denial)?;
        self.remaining = next_cardinality;
        self.remaining_items = remaining_items;
        self.remaining_retained_representation_bytes = remaining_retained_representation_bytes;
        Ok(())
    }

    pub(super) fn charge_representation_only(
        &mut self,
        retained_representation_bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let remaining = self
            .remaining_retained_representation_bytes
            .checked_sub(retained_representation_bytes)
            .ok_or_else(reservation_denial)?;
        self.remaining_retained_representation_bytes = remaining;
        Ok(())
    }
}

fn total(cardinality: ApplicationCandidateCardinalityCeiling) -> Option<usize> {
    cardinality
        .maximum_creates()
        .checked_add(cardinality.maximum_deletes())?
        .checked_add(cardinality.maximum_links())?
        .checked_add(cardinality.maximum_unlinks())?
        .checked_add(cardinality.maximum_writes())?
        .checked_add(cardinality.maximum_emits())
}

fn capacity_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
        "candidate requirements exceed an installed or runtime ceiling",
    )
}

fn reservation_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
        "candidate writer exceeded its admitted reservation",
    )
}

#[cfg(test)]
#[path = "candidate_reservation_tests.rs"]
mod tests;

#[cfg(test)]
mod atomic_tests;
