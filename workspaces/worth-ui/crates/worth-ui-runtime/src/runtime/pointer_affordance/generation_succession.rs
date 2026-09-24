use super::{UiPointerAffordanceReuseBasis, UiPointerAffordanceSnapshot};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPointerAffordanceGenerationSuccessionDenial {
    ForeignSession,
    StalePredecessor,
    StaleObservation,
    ObservationTimeExpired,
}

/// Prepared pointer-owner output for retained mounted identities. Embedded
/// standing decisions are observed anew; predecessor decisions are never relabeled.
pub(crate) struct UiPreparedPointerAffordanceGenerationSuccession {
    predecessor: WorthUiActiveApplicationGenerationIdentity,
    successor: WorthUiActiveApplicationGenerationIdentity,
    snapshot: Option<UiPointerAffordanceSnapshot>,
    changed_surfaces: Box<[UiSemanticSurfaceIdentity]>,
    bindings: Box<
        [(
            UiSemanticSurfaceIdentity,
            Option<worth_ui_host_contract::UiHostObservationPresentationBasis>,
        )],
    >,
    predecessor_observation: Option<super::UiPointerAffordanceSnapshot>,
    observed_millis: Option<u64>,
}

impl UiPreparedPointerAffordanceGenerationSuccession {
    pub(crate) fn prepare(
        predecessor: WorthUiActiveApplicationGenerationIdentity,
        successor: WorthUiActiveApplicationGenerationIdentity,
        succession: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationSuccession,
        source_basis: u64,
        previous: Option<&UiPointerAffordanceSnapshot>,
        observed_millis: Option<u64>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        observe: impl FnMut(
            crate::runtime::interaction::UiPresentedInteractionTargetView,
        ) -> Result<
            crate::runtime::intent::UiIntentStandingOperabilityObservation,
            crate::runtime::intent::UiIntentStandingOperabilityUnavailable,
        >,
    ) -> Result<Self, UiPointerAffordanceGenerationSuccessionDenial> {
        if predecessor.session_identity() != successor.session_identity() {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::ForeignSession);
        }
        if predecessor.prepared_generation() != succession.predecessor()
            || successor.prepared_generation() != succession.successor()
        {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::StalePredecessor);
        }
        if previous.is_some_and(|snapshot| snapshot.generation() != &predecessor) {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::StalePredecessor);
        }
        let surfaces: Vec<_> = previous
            .into_iter()
            .flat_map(|snapshot| snapshot.projections())
            .map(|row| row.surface())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let snapshot = previous.map(|snapshot| {
            snapshot.observe_generation_successor(successor.clone(), source_basis, observe)
        });
        let bindings = surfaces
            .iter()
            .map(|surface| {
                (
                    *surface,
                    mounted
                        .current_presentation_for_surface(*surface)
                        .map(|displayed| displayed.basis()),
                )
            })
            .collect();
        let changed_surfaces = surfaces
            .into_iter()
            .filter(|surface| {
                mounted
                    .current_presentation_for_surface(*surface)
                    .map(|displayed| displayed.basis())
                    .is_some_and(|current| {
                        mounted.pointer_presentation_pending(
                            *surface,
                            current.binding(),
                            snapshot.as_ref(),
                        )
                    })
            })
            .collect();
        Ok(Self {
            predecessor,
            successor,
            snapshot,
            changed_surfaces,
            bindings,
            predecessor_observation: previous.cloned(),
            observed_millis,
        })
    }

    pub(crate) fn predecessor(&self) -> &WorthUiActiveApplicationGenerationIdentity {
        &self.predecessor
    }
    pub(crate) fn successor(&self) -> &WorthUiActiveApplicationGenerationIdentity {
        &self.successor
    }
    pub(crate) fn snapshot(&self) -> Option<&UiPointerAffordanceSnapshot> {
        self.snapshot.as_ref()
    }
    pub(crate) fn into_snapshot(self) -> Option<UiPointerAffordanceSnapshot> {
        self.snapshot
    }

    pub(crate) fn validate_predecessor(
        &self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        snapshot: Option<&UiPointerAffordanceSnapshot>,
        now_millis: Option<u64>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<(), UiPointerAffordanceGenerationSuccessionDenial> {
        if generation != &self.predecessor
            || self.bindings.iter().any(|(surface, presentation)| {
                mounted
                    .current_presentation_for_surface(*surface)
                    .map(|displayed| displayed.basis())
                    != *presentation
            })
        {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::StalePredecessor);
        }
        if !match (&self.predecessor_observation, snapshot) {
            (None, None) => true,
            (Some(previous), Some(current)) => previous.same_owner_snapshot(current),
            _ => false,
        } {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::StaleObservation);
        }
        if self
            .observed_millis
            .zip(now_millis)
            .is_some_and(|(old, now)| now < old)
            || self
                .snapshot()
                .and_then(UiPointerAffordanceSnapshot::confirmation_deadline)
                .is_some_and(|deadline| now_millis.is_none_or(|now| now >= deadline))
        {
            return Err(UiPointerAffordanceGenerationSuccessionDenial::ObservationTimeExpired);
        }
        Ok(())
    }

    pub(crate) fn requires_timed_publication(&self) -> bool {
        self.snapshot()
            .and_then(UiPointerAffordanceSnapshot::confirmation_deadline)
            .is_some()
    }

    pub(crate) fn reuse_basis(
        &self,
        includes: impl Fn(UiSemanticSurfaceIdentity) -> bool,
    ) -> UiPointerAffordanceReuseBasis {
        UiPointerAffordanceReuseBasis::from_snapshot(self.snapshot(), includes)
    }

    pub(crate) fn changed_surfaces(&self) -> &[UiSemanticSurfaceIdentity] {
        &self.changed_surfaces
    }

    pub(crate) fn affected_targets(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiMountedInstanceIdentity> + '_ {
        self.predecessor_observation
            .iter()
            .chain(self.snapshot.iter())
            .flat_map(UiPointerAffordanceSnapshot::projections)
            .filter(|row| {
                self.changed_surfaces.contains(&row.surface()) || self.requires_timed_publication()
            })
            .filter_map(super::UiPointerAffordanceProjection::target)
    }
}
