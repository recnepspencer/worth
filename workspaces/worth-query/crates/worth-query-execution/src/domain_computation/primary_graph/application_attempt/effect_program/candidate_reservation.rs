use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateContract,
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
    remaining: ApplicationCandidateCardinalityCeiling,
    remaining_retained_representation_bytes: usize,
}

impl WorthQueryCandidateReservation {
    pub(super) fn admit(
        requested: ApplicationCandidateRequirements,
        contract: ApplicationCandidateContract,
        candidate_item_capacity: u64,
        retained_representation_byte_capacity: u64,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let requested_total = total(requested.cardinality()).ok_or_else(capacity_denial)?;
        let within_runtime = u64::try_from(requested_total)
            .is_ok_and(|n| n <= candidate_item_capacity)
            && u64::try_from(
                requested
                    .resources()
                    .maximum_retained_representation_bytes(),
            )
            .is_ok_and(|n| n <= retained_representation_byte_capacity);
        if !within_runtime {
            return Err(capacity_denial());
        }
        Self::admit_contract(requested, contract)
    }

    /// Logical counters only. Query-owned fixed programs have already admitted
    /// their resource contract through native graph-work support. A handler
    /// must instead enter `admit` with its real runtime profile before funding.
    pub(super) fn admit_contract(
        requested: ApplicationCandidateRequirements,
        contract: ApplicationCandidateContract,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let requested_cardinality = requested.cardinality();
        total(requested_cardinality).ok_or_else(capacity_denial)?;
        let within_binding = match contract {
            ApplicationCandidateContract::FixedShape(ceiling) => {
                within_counts(requested_cardinality, ceiling.cardinality())
                    && requested
                        .resources()
                        .maximum_retained_representation_bytes()
                        <= ceiling.resources().maximum_retained_representation_bytes()
            }
            ApplicationCandidateContract::FixedCardinality(ceiling) => {
                within_counts(requested_cardinality, ceiling)
            }
            ApplicationCandidateContract::Prepared => true,
        };
        if !within_binding {
            return Err(capacity_denial());
        }
        Ok(Self {
            remaining: requested_cardinality,
            remaining_retained_representation_bytes: requested
                .resources()
                .maximum_retained_representation_bytes(),
        })
    }

    pub(super) fn total_items(&self) -> usize {
        total(self.remaining).expect("admitted reservation counts remain representable")
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
        self.remaining_retained_representation_bytes = remaining_retained_representation_bytes;
        Ok(())
    }

    pub(super) fn check_representation_only(
        &self,
        bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.remaining_retained_representation_bytes
            .checked_sub(bytes)
            .map(|_| ())
            .ok_or_else(reservation_denial)
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

fn within_counts(
    requested: ApplicationCandidateCardinalityCeiling,
    ceiling: ApplicationCandidateCardinalityCeiling,
) -> bool {
    requested.maximum_creates() <= ceiling.maximum_creates()
        && requested.maximum_deletes() <= ceiling.maximum_deletes()
        && requested.maximum_links() <= ceiling.maximum_links()
        && requested.maximum_unlinks() <= ceiling.maximum_unlinks()
        && requested.maximum_writes() <= ceiling.maximum_writes()
        && requested.maximum_emits() <= ceiling.maximum_emits()
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
