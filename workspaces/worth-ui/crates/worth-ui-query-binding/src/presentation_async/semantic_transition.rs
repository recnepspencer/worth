use std::collections::{HashMap, HashSet};

use worth_ui_host_contract::{
    UiHostPresentationLineageIdentity, UiHostSurfaceIdentity, UiMountedFrameIdentity,
    UiMountedPaintCommandIdentity, UiSemanticSurfaceIdentity,
};

use super::{WorthUiPresentationMechanicBasis, WorthUiPresentationRequestBasis};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct PresentationLineageKey {
    semantic_surface: UiSemanticSurfaceIdentity,
    host_lineage: UiHostPresentationLineageIdentity,
}

#[derive(Clone)]
pub(super) struct RetainedPresentationSemanticState {
    frame: UiMountedFrameIdentity,
    predecessor: Option<UiMountedFrameIdentity>,
    host_surface: UiHostSurfaceIdentity,
    mechanics: HashMap<UiMountedPaintCommandIdentity, WorthUiPresentationMechanicBasis>,
    pins: HashSet<super::WorthUiPresentationPinBasis>,
}

pub(super) struct PresentationSemanticTransition {
    successor: RetainedPresentationSemanticState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PresentationSemanticTransitionDenial {
    MissingBaseline,
    StalePredecessor,
    ForeignHostSurface,
    UnknownRemovedMechanic,
    UnknownReleasedPin,
}

impl PresentationLineageKey {
    pub(super) fn from_basis(basis: &WorthUiPresentationRequestBasis) -> Self {
        Self {
            semantic_surface: basis.semantic_surface(),
            host_lineage: basis.host_lineage(),
        }
    }
}

impl PresentationSemanticTransition {
    pub(super) fn plan(
        current: Option<&RetainedPresentationSemanticState>,
        basis: &WorthUiPresentationRequestBasis,
    ) -> Result<Self, PresentationSemanticTransitionDenial> {
        validate_predecessor(current, basis)?;
        Self::build(current, basis)
    }

    pub(super) fn plan_reconstruction(
        unresolved: &RetainedPresentationSemanticState,
        basis: &WorthUiPresentationRequestBasis,
    ) -> Result<Self, PresentationSemanticTransitionDenial> {
        if !basis.complete() || basis.predecessor() != unresolved.predecessor {
            return Err(PresentationSemanticTransitionDenial::StalePredecessor);
        }
        Self::build(Some(unresolved), basis)
    }

    fn build(
        current: Option<&RetainedPresentationSemanticState>,
        basis: &WorthUiPresentationRequestBasis,
    ) -> Result<Self, PresentationSemanticTransitionDenial> {
        let mut mechanics = if basis.complete() {
            HashMap::new()
        } else {
            current
                .map(|state| state.mechanics.clone())
                .ok_or(PresentationSemanticTransitionDenial::MissingBaseline)?
        };
        for removed in basis.removed_mechanics() {
            if basis.complete() {
                if current.is_none_or(|state| !state.mechanics.contains_key(removed)) {
                    return Err(PresentationSemanticTransitionDenial::UnknownRemovedMechanic);
                }
                continue;
            }
            let Some(_) = mechanics.remove(removed) else {
                return Err(PresentationSemanticTransitionDenial::UnknownRemovedMechanic);
            };
        }
        for mechanic in basis.mechanics() {
            mechanics.insert(mechanic.mechanic(), mechanic.clone());
        }
        let prior_pins = current.map(|state| &state.pins);
        for released in basis.pin_releases() {
            if prior_pins.is_none_or(|pins| !pins.contains(released)) {
                return Err(PresentationSemanticTransitionDenial::UnknownReleasedPin);
            }
        }
        let successor = RetainedPresentationSemanticState {
            frame: basis.mounted_frame(),
            predecessor: basis.predecessor(),
            host_surface: basis.host_surface(),
            mechanics,
            pins: basis.binding_pins().iter().copied().collect(),
        };
        Ok(Self { successor })
    }

    pub(super) fn successor(&self) -> &RetainedPresentationSemanticState {
        &self.successor
    }
}

fn validate_predecessor(
    current: Option<&RetainedPresentationSemanticState>,
    basis: &WorthUiPresentationRequestBasis,
) -> Result<(), PresentationSemanticTransitionDenial> {
    let Some(current) = current else {
        if basis.predecessor().is_some() || !basis.complete() {
            return Err(PresentationSemanticTransitionDenial::MissingBaseline);
        }
        return Ok(());
    };
    if basis.predecessor() != Some(current.frame) {
        return Err(PresentationSemanticTransitionDenial::StalePredecessor);
    }
    if basis.host_surface() != current.host_surface {
        return Err(PresentationSemanticTransitionDenial::ForeignHostSurface);
    }
    Ok(())
}
