//! Borrow the exact retained placement under its original state gate.
use super::{EnvelopeBacking, EnvelopeState, RelationalCommitEnvelopeResidency};
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::history::data::PositionedCanonicalCommit;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailable;
use std::sync::MutexGuard;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCauseOwnerPort as _;

pub(crate) struct EnvelopeArtifactLoan<'a>(MutexGuard<'a, EnvelopeState>);

impl RelationalCommitEnvelopeResidency {
    pub(crate) fn loan_artifact_source(&self) -> EnvelopeArtifactLoan<'_> {
        EnvelopeArtifactLoan(self.state.lock().unwrap_or_else(|error| error.into_inner()))
    }
}

impl EnvelopeArtifactLoan<'_> {
    pub(crate) fn backing(&self) -> &EnvelopeBacking {
        &self.0.backing
    }

    pub(crate) fn damage(&self) -> Option<RelationalStorageUnavailable> {
        self.0.damage
    }

    pub(crate) fn position(&self) -> PatchStreamPosition {
        self.0
            .position
            .as_ref()
            .map_or(PatchStreamPosition(0), |position| position.position())
    }

    pub(crate) fn resident_position(&self) -> Option<PositionedCanonicalCommit> {
        self.0
            .position
            .as_ref()
            .and_then(|position| position.resident())
    }

    pub(crate) fn remember_damage(&mut self, failure: RelationalStorageUnavailable) {
        if failure.cause.is_proven_damage() {
            self.0.damage = Some(failure);
        }
    }

    pub(crate) fn remember_original_failure(&mut self, stop: &RelationalRecoveryStop) {
        self.0.remember_original_failure(stop);
    }
}
