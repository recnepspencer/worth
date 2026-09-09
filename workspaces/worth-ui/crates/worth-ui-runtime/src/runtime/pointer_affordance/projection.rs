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

    pub(super) fn same_mechanic(&self, other: &Self) -> bool {
        self.surface == other.surface
            && self.pointer == other.pointer
            && self.target() == other.target()
            && self.family() == other.family()
    }
}
