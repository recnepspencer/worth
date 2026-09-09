use std::collections::{BTreeMap, BTreeSet};

use worth_ui_host_contract::{UiMountedPresentationAttemptIdentity, UiSemanticSurfaceIdentity};

type OwnerMap = BTreeMap<
    UiSemanticSurfaceIdentity,
    crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle,
>;

pub(super) type BackdropProjections = BTreeMap<
    crate::runtime::overlay_composition::UiBackdropInstanceIdentity,
    crate::runtime::appearance::UiBackdropAppearanceProjection,
>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::facade::entry) struct UiActiveBackdropAppearanceWork {
    pub(in crate::facade::entry) candidates_visited: usize,
    pub(in crate::facade::entry) roles_resolved: usize,
    pub(in crate::facade::entry) portal_records_looked_up: usize,
    pub(in crate::facade::entry) motion_targets_looked_up: usize,
    pub(in crate::facade::entry) retained_participants_visited: usize,
    pub(in crate::facade::entry) motion_declarations_visited: usize,
    pub(in crate::facade::entry) motion_bindings_visited: usize,
    pub(in crate::facade::entry) source_comparison_steps: usize,
    pub(in crate::facade::entry) source_changed_keys_visited: usize,
    pub(in crate::facade::entry) portal_rows_serialized: usize,
    pub(in crate::facade::entry) portal_source_rows_visited: usize,
}

#[derive(Default)]
pub(super) struct UiActiveBackdropAppearanceCandidate {
    pub(super) projections: BackdropProjections,
    pub(super) work: UiActiveBackdropAppearanceWork,
    pub(super) sources: Option<super::local_sources::UiActiveOverlaySourceSnapshot>,
}

pub(in crate::facade::entry) struct UiActiveOverlayCompositionOwners {
    current: OwnerMap,
    backdrops: BTreeMap<UiSemanticSurfaceIdentity, UiActiveBackdropAppearanceCandidate>,
    pending: BTreeMap<UiMountedPresentationAttemptIdentity, UiActiveOverlayRetentionCandidate>,
}

pub(super) struct UiActiveOverlayRetentionCandidate {
    requested_surfaces: BTreeSet<UiSemanticSurfaceIdentity>,
    active_surfaces: BTreeSet<UiSemanticSurfaceIdentity>,
    updates: Vec<UiActiveOverlayOwnerUpdate>,
    backdrops: BTreeMap<UiSemanticSurfaceIdentity, UiActiveBackdropAppearanceCandidate>,
}

pub(super) enum UiActiveOverlayOwnerUpdate {
    Replace {
        surface: UiSemanticSurfaceIdentity,
        owner: crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle,
    },
    Advance {
        surface: UiSemanticSurfaceIdentity,
        prepared: crate::runtime::overlay_composition::UiPreparedOverlayComposition,
    },
}

impl UiActiveOverlayCompositionOwners {
    pub(in crate::facade::entry) fn new() -> Self {
        Self {
            current: BTreeMap::new(),
            backdrops: BTreeMap::new(),
            pending: BTreeMap::new(),
        }
    }

    pub(super) fn current(&self) -> &OwnerMap {
        &self.current
    }

    pub(super) fn sources(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<&super::local_sources::UiActiveOverlaySourceSnapshot> {
        self.backdrops
            .get(&surface)
            .and_then(|candidate| candidate.sources.as_ref())
    }

    pub(super) fn backdrop_projections(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<&BackdropProjections> {
        self.backdrops
            .get(&surface)
            .map(|candidate| &candidate.projections)
    }

    #[cfg(test)]
    pub(in crate::facade::entry) fn backdrop_work_for_test(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<UiActiveBackdropAppearanceWork> {
        self.backdrops.get(&surface).map(|candidate| candidate.work)
    }

    pub(super) fn stage(
        &mut self,
        attempt: UiMountedPresentationAttemptIdentity,
        candidate: UiActiveOverlayRetentionCandidate,
    ) -> Result<(), ()> {
        if self.pending.insert(attempt, candidate).is_some() {
            return Err(());
        }
        Ok(())
    }

    pub(in crate::facade::entry) fn settle(
        &mut self,
        outcome: &crate::mounting::UiMountedFrameOutcome,
    ) {
        use crate::mounting::UiMountedFrameOutcome as Outcome;
        match outcome {
            Outcome::Published(receipt)
            | Outcome::Unchanged(receipt)
            | Outcome::Reconciled(receipt) => self.commit(receipt.attempt()),
            Outcome::RejectedBeforeEffects(rejected) => self.discard(rejected.attempt()),
            Outcome::PresentationIndeterminate(indeterminate) => {
                self.discard(indeterminate.report().attempt())
            }
            Outcome::Superseded(superseded) => self.discard(superseded.attempt()),
            Outcome::AdmissionDenied(rejection) => {
                if let Some(attempt) = rejection.attempt() {
                    self.discard(attempt);
                }
            }
            Outcome::InFlight(_) | Outcome::RetentionDenied(_) | Outcome::CompletionDenied(_) => {}
        }
    }

    #[cfg(test)]
    pub(in crate::facade::entry) fn current_for_test(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<&crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle> {
        self.current.get(&surface)
    }

    fn commit(&mut self, attempt: UiMountedPresentationAttemptIdentity) {
        let Some(candidate) = self.pending.remove(&attempt) else {
            return;
        };
        self.current.retain(|surface, _| {
            !candidate.requested_surfaces.contains(surface)
                || candidate.active_surfaces.contains(surface)
        });
        self.backdrops.retain(|surface, _| {
            !candidate.requested_surfaces.contains(surface)
                || candidate.active_surfaces.contains(surface)
        });
        self.backdrops.extend(candidate.backdrops);
        for update in candidate.updates {
            match update {
                UiActiveOverlayOwnerUpdate::Replace { surface, owner } => {
                    self.current.insert(surface, owner);
                }
                UiActiveOverlayOwnerUpdate::Advance { surface, prepared } => self
                    .current
                    .get_mut(&surface)
                    .expect("prepared overlay successor retains its exact predecessor owner")
                    .retain_prepared(prepared)
                    .expect("accepted overlay successor retains its exact prepared predecessor"),
            }
        }
    }

    fn discard(&mut self, attempt: UiMountedPresentationAttemptIdentity) {
        self.pending.remove(&attempt);
    }
}

impl UiActiveOverlayOwnerUpdate {
    pub(super) fn snapshot(
        &self,
    ) -> Option<&crate::runtime::overlay_composition::UiOverlayStackSnapshot> {
        match self {
            Self::Replace { owner, .. } => owner.current(),
            Self::Advance { prepared, .. } => Some(prepared.snapshot()),
        }
    }
}

impl UiActiveOverlayRetentionCandidate {
    pub(super) fn new(
        requested_surfaces: BTreeSet<UiSemanticSurfaceIdentity>,
        active_surfaces: BTreeSet<UiSemanticSurfaceIdentity>,
        updates: Vec<UiActiveOverlayOwnerUpdate>,
        backdrops: BTreeMap<UiSemanticSurfaceIdentity, UiActiveBackdropAppearanceCandidate>,
    ) -> Self {
        Self {
            requested_surfaces,
            active_surfaces,
            updates,
            backdrops,
        }
    }
}
