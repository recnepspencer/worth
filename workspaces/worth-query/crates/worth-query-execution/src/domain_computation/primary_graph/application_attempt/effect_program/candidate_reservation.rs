use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
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
    validator_work: WorthQueryCandidateValidatorWorkAdmission,
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryCandidateValidatorWorkAdmission {
    UnreservedInternal,
    Reserved { maximum_work: usize },
}

impl WorthQueryCandidateReservation {
    pub(super) fn admit(
        requested: ApplicationCandidateRequirements,
        ceiling: ApplicationCandidateRequirements,
        candidate_item_capacity: u64,
        retained_representation_byte_capacity: u64,
        validator_work_capacity: u64,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let requested_cardinality = requested.cardinality();
        let ceiling_cardinality = ceiling.cardinality();
        let requested_total = total(requested_cardinality).ok_or_else(capacity_denial)?;
        let within_binding = requested_cardinality.maximum_creates()
            <= ceiling_cardinality.maximum_creates()
            && requested_cardinality.maximum_deletes() <= ceiling_cardinality.maximum_deletes()
            && requested_cardinality.maximum_links() <= ceiling_cardinality.maximum_links()
            && requested_cardinality.maximum_unlinks() <= ceiling_cardinality.maximum_unlinks()
            && requested_cardinality.maximum_writes() <= ceiling_cardinality.maximum_writes()
            && requested_cardinality.maximum_emits() <= ceiling_cardinality.maximum_emits()
            && requested
                .resources()
                .maximum_retained_representation_bytes()
                <= ceiling.resources().maximum_retained_representation_bytes()
            && requested.resources().maximum_validator_work()
                <= ceiling.resources().maximum_validator_work();
        let within_runtime = u64::try_from(requested_total)
            .is_ok_and(|count| count <= candidate_item_capacity)
            && u64::try_from(
                requested
                    .resources()
                    .maximum_retained_representation_bytes(),
            )
            .is_ok_and(|bytes| bytes <= retained_representation_byte_capacity)
            && u64::try_from(requested.resources().maximum_validator_work())
                .is_ok_and(|work| work <= validator_work_capacity);
        if !within_binding || !within_runtime {
            return Err(capacity_denial());
        }
        Ok(Self {
            remaining: requested_cardinality,
            remaining_retained_representation_bytes: requested
                .resources()
                .maximum_retained_representation_bytes(),
            validator_work: WorthQueryCandidateValidatorWorkAdmission::Reserved {
                maximum_work: requested.resources().maximum_validator_work(),
            },
        })
    }

    pub(super) fn total_items(&self) -> usize {
        total(self.remaining).unwrap_or(usize::MAX)
    }

    pub(super) const fn validator_work_admission(
        &self,
    ) -> WorthQueryCandidateValidatorWorkAdmission {
        self.validator_work
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

impl WorthQueryCandidateValidatorWorkAdmission {
    pub(in crate::domain_computation::primary_graph) const fn unreserved_internal() -> Self {
        Self::UnreservedInternal
    }

    pub(in crate::domain_computation::primary_graph) const fn maximum_work(self) -> Option<usize> {
        match self {
            Self::UnreservedInternal => None,
            Self::Reserved { maximum_work } => Some(maximum_work),
        }
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
