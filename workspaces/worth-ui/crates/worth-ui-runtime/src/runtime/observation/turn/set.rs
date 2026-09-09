use super::{UiAdmittedObservation, UiObservationSetSummary, UiObservationTurnIdentity};

pub struct UiAdmittedObservationSet {
    turn: UiObservationTurnIdentity,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    source_basis: u64,
    observations: Box<[UiAdmittedObservation]>,
    summary: UiObservationSetSummary,
    appearance_owner_snapshot: Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pointer_snapshot: Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
    _lease: super::super::resource_ledger::UiObservationSetLease,
}

impl UiAdmittedObservationSet {
    pub(super) fn seal(
        turn: UiObservationTurnIdentity,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
        observations: Box<[UiAdmittedObservation]>,
        retained_bytes: usize,
        appearance_owner_snapshot: Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
        pointer_snapshot: Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        lease: super::super::resource_ledger::UiObservationSetLease,
    ) -> Self {
        debug_assert!(!observations.is_empty() || pointer_snapshot.is_some());
        let families = observations
            .iter()
            .map(UiAdmittedObservation::family)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let summary = UiObservationSetSummary::new(observations.len(), retained_bytes, families);
        Self {
            turn,
            session,
            source_basis,
            observations,
            summary,
            appearance_owner_snapshot,
            pointer_snapshot,
            _lease: lease,
        }
    }

    pub(crate) fn pointer_snapshot(
        &self,
    ) -> Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot> {
        self.pointer_snapshot.as_ref()
    }
    pub(crate) fn take_pointer_snapshot(
        &mut self,
    ) -> Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot> {
        self.pointer_snapshot.take()
    }

    pub const fn turn(&self) -> UiObservationTurnIdentity {
        self.turn
    }

    pub fn observations(&self) -> &[UiAdmittedObservation] {
        &self.observations
    }

    pub const fn retained_bytes(&self) -> usize {
        self.summary.retained_bytes()
    }

    pub const fn summary(&self) -> &UiObservationSetSummary {
        &self.summary
    }

    pub(crate) fn take_appearance_owner_snapshot(
        &mut self,
    ) -> Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.take()
    }

    pub(crate) const fn appearance_owner_snapshot(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.as_ref()
    }

    #[cfg(test)]
    pub(crate) const fn carries_appearance_owner_snapshot_for_test(&self) -> bool {
        self.appearance_owner_snapshot.is_some()
    }

    #[cfg(test)]
    pub(crate) const fn appearance_owner_snapshot_for_test(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.as_ref()
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }

    pub(crate) const fn source_basis(&self) -> u64 {
        self.source_basis
    }

    pub(in crate::runtime::observation) fn into_observations(self) -> Box<[UiAdmittedObservation]> {
        self.observations
    }
}
