use crate::runtime::intent::{
    UiIntentStandingOperabilityObservation, UiIntentStandingOperabilityUnavailable,
};
use worth_ui_host_contract::{
    UiHostPointerIdentity, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

pub(crate) struct UiPointerAffordanceProjection {
    surface: UiSemanticSurfaceIdentity,
    pointer: UiHostPointerIdentity,
    target: PointerTarget,
}

enum PointerTarget {
    Outside,
    Inside {
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
        operability:
            Result<UiIntentStandingOperabilityObservation, UiIntentStandingOperabilityUnavailable>,
    },
}

impl UiPointerAffordanceProjection {
    pub(super) fn observe_successor(
        &self,
        observe: &mut impl FnMut(
            crate::runtime::interaction::UiPresentedInteractionTargetView,
        ) -> Result<
            UiIntentStandingOperabilityObservation,
            UiIntentStandingOperabilityUnavailable,
        >,
    ) -> Self {
        let target = match &self.target {
            PointerTarget::Outside => PointerTarget::Outside,
            PointerTarget::Inside { target, .. } => PointerTarget::Inside {
                target: *target,
                operability: observe(*target),
            },
        };
        Self {
            surface: self.surface,
            pointer: self.pointer,
            target,
        }
    }

    pub(super) fn from_primary(
        surface: UiSemanticSurfaceIdentity,
        posture: crate::runtime::interaction::UiPointerPresenceAppearancePosture,
        observe: impl FnOnce(
            crate::runtime::interaction::UiPresentedInteractionTargetView,
        ) -> Result<
            UiIntentStandingOperabilityObservation,
            UiIntentStandingOperabilityUnavailable,
        >,
    ) -> Self {
        let target = match posture.presented_target() {
            Some(target) => PointerTarget::Inside {
                target,
                operability: observe(target),
            },
            None => PointerTarget::Outside,
        };
        Self {
            surface,
            pointer: posture.pointer(),
            target,
        }
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) const fn pointer(&self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub(crate) fn target(&self) -> Option<UiMountedInstanceIdentity> {
        match &self.target {
            PointerTarget::Outside => None,
            PointerTarget::Inside { target, .. } => Some(target.mounted_instance()),
        }
    }
    pub(crate) fn presented_target(
        &self,
    ) -> Option<crate::runtime::interaction::UiPresentedInteractionTargetView> {
        match &self.target {
            PointerTarget::Outside => None,
            PointerTarget::Inside { target, .. } => Some(*target),
        }
    }
    pub(crate) fn family(&self) -> crate::declaration::UiPointerAffordance {
        match &self.target {
            PointerTarget::Inside {
                operability: Ok(observation),
                ..
            } if observation.is_operable() => crate::declaration::UiPointerAffordance::Activation,
            _ => crate::declaration::UiPointerAffordance::Default,
        }
    }
    /// The affordance this observation decided for its target, or `None`
    /// when a publication in flight withheld the operability decision. The
    /// next observation turn after it settles decides again, so a withheld
    /// reading is still owed.
    pub(crate) fn decided_family(
        &self,
    ) -> Option<worth_ui_host_contract::UiPointerAffordanceFamily> {
        use worth_ui_host_contract::UiPointerAffordanceFamily as Family;
        if let PointerTarget::Inside {
            operability: Err(unavailable),
            ..
        } = &self.target
        {
            if unavailable.is_withheld_by_publication() {
                return None;
            }
        }
        Some(match self.family() {
            crate::declaration::UiPointerAffordance::Default => Family::Default,
            crate::declaration::UiPointerAffordance::Activation => Family::Activation,
        })
    }

    /// The affordance to present for this observation. Until a withheld
    /// decision is made, the target keeps the affordance already `published`
    /// for it; claiming the default instead flickers the cursor off a target
    /// that has not changed.
    pub(crate) fn resolved_family(
        &self,
        published: Option<worth_ui_host_contract::UiPointerAffordanceFamily>,
    ) -> worth_ui_host_contract::UiPointerAffordanceFamily {
        self.decided_family()
            .or(published)
            .unwrap_or(worth_ui_host_contract::UiPointerAffordanceFamily::Default)
    }

    pub(crate) fn operability(
        &self,
    ) -> Option<
        Result<&UiIntentStandingOperabilityObservation, &UiIntentStandingOperabilityUnavailable>,
    > {
        match &self.target {
            PointerTarget::Outside => None,
            PointerTarget::Inside { operability, .. } => Some(operability.as_ref()),
        }
    }
    pub(super) fn confirmation_deadline(&self) -> Option<u64> {
        self.operability()?.ok()?.confirmation_deadline()
    }

    #[cfg(test)]
    pub(super) fn same_mechanic(&self, other: &Self) -> bool {
        self.surface == other.surface
            && self.pointer == other.pointer
            && self.target() == other.target()
            && self.family() == other.family()
    }
}
