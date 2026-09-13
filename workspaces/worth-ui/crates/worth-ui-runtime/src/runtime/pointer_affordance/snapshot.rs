use super::UiPointerAffordanceProjection;
use crate::runtime::observation::{UiObservationTurnCloseAuthority, UiObservationTurnIdentity};
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

#[derive(Clone)]
pub(crate) struct UiPointerAffordanceSnapshot {
    turn: UiObservationTurnIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    source_basis: u64,
    projections: std::rc::Rc<[UiPointerAffordanceProjection]>,
    invalidated_surfaces: std::rc::Rc<std::collections::BTreeSet<UiSemanticSurfaceIdentity>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerAffordanceObservationIdentity {
    turn: UiObservationTurnIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    source_basis: u64,
}

impl UiPointerAffordanceSnapshot {
    pub(crate) fn same_owner_snapshot(&self, other: &Self) -> bool {
        self.observation_identity() == other.observation_identity()
            && std::rc::Rc::ptr_eq(&self.projections, &other.projections)
            && self.invalidated_surfaces == other.invalidated_surfaces
    }
    pub(super) fn observe_generation_successor(
        &self,
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        source_basis: u64,
        mut observe: impl FnMut(
            crate::runtime::interaction::UiPresentedInteractionTargetView,
        ) -> Result<
            crate::runtime::intent::UiIntentStandingOperabilityObservation,
            crate::runtime::intent::UiIntentStandingOperabilityUnavailable,
        >,
    ) -> Self {
        Self {
            turn: self.turn,
            generation,
            source_basis,
            projections: self
                .active_projections()
                .map(|projection| projection.observe_successor(&mut observe))
                .collect::<Vec<_>>()
                .into(),
            invalidated_surfaces: std::rc::Rc::new(std::collections::BTreeSet::new()),
        }
    }

    pub(crate) fn observation_identity(&self) -> UiPointerAffordanceObservationIdentity {
        UiPointerAffordanceObservationIdentity {
            turn: self.turn,
            generation: self.generation.clone(),
            source_basis: self.source_basis,
        }
    }

    pub(crate) fn seal_pointer_at_turn_close(
        _authority: &UiObservationTurnCloseAuthority,
        turn: UiObservationTurnIdentity,
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        source_basis: u64,
        presence: &crate::runtime::interaction::UiPointerPresenceAppearanceOwnerSnapshot,
        mut observe: impl FnMut(
            crate::runtime::interaction::UiPresentedInteractionTargetView,
        ) -> Result<
            crate::runtime::intent::UiIntentStandingOperabilityObservation,
            crate::runtime::intent::UiIntentStandingOperabilityUnavailable,
        >,
    ) -> Self {
        let projections = presence
            .primary_postures()
            .map(|(surface, posture)| {
                UiPointerAffordanceProjection::from_primary(surface, posture, &mut observe)
            })
            .collect::<Vec<_>>()
            .into();
        Self {
            turn,
            generation,
            source_basis,
            projections,
            invalidated_surfaces: std::rc::Rc::new(std::collections::BTreeSet::new()),
        }
    }
    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }
    pub(crate) fn observation_turn(&self) -> u64 {
        self.turn.as_u64()
    }
    pub(crate) fn source_basis(&self) -> u64 {
        self.source_basis
    }
    pub(crate) fn projections(&self) -> &[UiPointerAffordanceProjection] {
        &self.projections
    }

    pub(crate) fn active_projections(
        &self,
    ) -> impl Iterator<Item = &UiPointerAffordanceProjection> {
        self.projections
            .iter()
            .filter(|projection| !self.invalidated_surfaces.contains(&projection.surface()))
    }

    pub(crate) fn invalidate_surface(&mut self, surface: UiSemanticSurfaceIdentity) {
        std::rc::Rc::make_mut(&mut self.invalidated_surfaces).insert(surface);
    }
    pub(crate) fn surface_is_invalidated(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.invalidated_surfaces.contains(&surface)
    }

    pub(crate) fn confirmation_deadline(&self) -> Option<u64> {
        self.active_projections()
            .filter_map(UiPointerAffordanceProjection::confirmation_deadline)
            .min()
    }

    #[cfg(test)]
    pub(crate) fn changed_surfaces(&self, previous: &Self) -> Box<[UiSemanticSurfaceIdentity]> {
        let previous_rows = previous
            .active_projections()
            .map(|row| (row.surface(), row))
            .collect::<std::collections::BTreeMap<_, _>>();
        let current_rows = self
            .active_projections()
            .map(|row| (row.surface(), row))
            .collect::<std::collections::BTreeMap<_, _>>();
        previous_rows
            .keys()
            .chain(current_rows.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(
                |surface| match (previous_rows.get(surface), current_rows.get(surface)) {
                    (Some(old), Some(new)) => !old.same_mechanic(new),
                    _ => true,
                },
            )
            .collect()
    }
}
