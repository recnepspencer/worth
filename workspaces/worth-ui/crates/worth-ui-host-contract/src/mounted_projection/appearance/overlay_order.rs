use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiOverlayParticipantIdentity {
    Portal(crate::UiMountedInstanceIdentity),
    Backdrop(super::UiMountedBackdropIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Inert, duplicate-checked host transport for one mounted overlay order.
///
/// This mechanic is not the runtime-issued semantic `UiOverlayStackSnapshot`
/// reserved for the Gate 1 overlay-composition owner. It grants no semantic
/// sealing or publication authority.
pub struct UiMountedOverlayOrderMechanic {
    semantic_surface: crate::UiSemanticSurfaceIdentity,
    presentation: crate::UiMountedPresentationAttemptIdentity,
    portal_revision: u64,
    backdrop_revision: u64,
    bottom_to_top: Box<[UiOverlayParticipantIdentity]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedOverlayOrderMechanicDenial {
    DuplicateParticipant(UiOverlayParticipantIdentity),
    MissingRevision,
}

impl UiMountedOverlayOrderMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_overlay_order(
        semantic_surface: crate::UiSemanticSurfaceIdentity,
        presentation: crate::UiMountedPresentationAttemptIdentity,
        portal_revision: u64,
        backdrop_revision: u64,
        bottom_to_top: impl IntoIterator<Item = UiOverlayParticipantIdentity>,
    ) -> Result<Self, UiMountedOverlayOrderMechanicDenial> {
        let bottom_to_top = bottom_to_top.into_iter().collect::<Vec<_>>();
        if !bottom_to_top.is_empty() && (portal_revision == 0 || backdrop_revision == 0) {
            return Err(UiMountedOverlayOrderMechanicDenial::MissingRevision);
        }
        let mut seen = BTreeSet::new();
        if let Some(duplicate) = bottom_to_top
            .iter()
            .find(|participant| !seen.insert((*participant).clone()))
        {
            return Err(UiMountedOverlayOrderMechanicDenial::DuplicateParticipant(
                duplicate.clone(),
            ));
        }
        Ok(Self {
            semantic_surface,
            presentation,
            portal_revision,
            backdrop_revision,
            bottom_to_top: bottom_to_top.into_boxed_slice(),
        })
    }

    pub const fn semantic_surface(&self) -> crate::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub const fn presentation(&self) -> crate::UiMountedPresentationAttemptIdentity {
        self.presentation
    }
    pub const fn portal_revision(&self) -> u64 {
        self.portal_revision
    }
    pub const fn backdrop_revision(&self) -> u64 {
        self.backdrop_revision
    }
    pub fn bottom_to_top(&self) -> &[UiOverlayParticipantIdentity] {
        &self.bottom_to_top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete(
        semantic_surface: crate::UiSemanticSurfaceIdentity,
        bottom_to_top: impl IntoIterator<Item = UiOverlayParticipantIdentity>,
    ) -> Result<UiMountedOverlayOrderMechanic, UiMountedOverlayOrderMechanicDenial> {
        UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            semantic_surface,
            crate::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            1,
            1,
            bottom_to_top,
        )
    }

    #[test]
    fn overlay_order_denies_an_exact_duplicate_backdrop_identity() {
        let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let identity = super::super::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            super::super::UiMountedBackdropScope::SurfaceSingleton(surface),
            1,
        )
        .unwrap();
        let duplicate = UiOverlayParticipantIdentity::Backdrop(identity.clone());

        assert_eq!(
            complete(surface, [duplicate.clone(), duplicate.clone()]),
            Err(UiMountedOverlayOrderMechanicDenial::DuplicateParticipant(
                duplicate
            ))
        );
    }

    #[test]
    fn overlay_order_accepts_one_declaration_for_two_portal_instances() {
        let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let first_portal = crate::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let second_portal = crate::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let first = super::super::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            super::super::UiMountedBackdropScope::PerPortalInstance(first_portal),
            1,
        )
        .unwrap();
        let second = super::super::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            super::super::UiMountedBackdropScope::PerPortalInstance(second_portal),
            1,
        )
        .unwrap();

        let order = complete(
            surface,
            [
                UiOverlayParticipantIdentity::Backdrop(first),
                UiOverlayParticipantIdentity::Backdrop(second),
            ],
        )
        .unwrap();
        assert_eq!(order.bottom_to_top().len(), 2);
    }

    #[test]
    fn overlay_order_distinguishes_a_rematerialized_surface_singleton() {
        let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let first = super::super::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            super::super::UiMountedBackdropScope::SurfaceSingleton(surface),
            1,
        )
        .unwrap();
        let rematerialized = super::super::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            super::super::UiMountedBackdropScope::SurfaceSingleton(surface),
            2,
        )
        .unwrap();
        assert_ne!(first, rematerialized);
    }
}
