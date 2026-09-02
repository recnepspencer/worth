#![allow(
    dead_code,
    reason = "Gate 1 retains the motion receipt seam for later overlay publication"
)]

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiMotionTargetIdentity {
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    owner_key: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionSemanticGeometry {
    components: [u32; 4],
    coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionGeometryDenial {
    NonFinite,
    NegativeExtent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiCommittedMotionPredecessorReceipt {
    geometry: Option<UiMotionSemanticGeometry>,
    visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiPreparedMotionSuccessorReceipt {
    target: UiMotionTargetIdentity,
    owner_revision: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    geometry: Option<UiMotionSemanticGeometry>,
    visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionTransitionRequest {
    predecessor: UiCommittedMotionPredecessorReceipt,
    successor: UiPreparedMotionSuccessorReceipt,
    declaration: super::UiMotionDeclaration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionTransitionRequestDenial {
    RevisionDidNotAdvance,
    SurfaceChanged,
    BindingChangedWithoutRebind,
    GeometryCoordinateSpaceChanged,
    /// This target's retained exit has issued physical work that has not
    /// settled. A successor transition would displace that retention while its
    /// terminal is still in flight, so the request is refused before effect.
    ExitRetentionAwaitingPhysicalSettlement,
}

impl UiMotionTargetIdentity {
    pub(crate) const fn from_family_owner(
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        owner_key: u64,
    ) -> Self {
        Self {
            semantic_surface,
            mounted_instance,
            owner_key,
        }
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) const fn mounted_instance(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn owner_key(self) -> u64 {
        self.owner_key
    }
}

impl UiMotionSemanticGeometry {
    pub(crate) fn from_committed_components(
        components: [f32; 4],
        coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace,
    ) -> Result<Self, UiMotionGeometryDenial> {
        if components.iter().any(|component| !component.is_finite()) {
            return Err(UiMotionGeometryDenial::NonFinite);
        }
        if components[2] < 0.0 || components[3] < 0.0 {
            return Err(UiMotionGeometryDenial::NegativeExtent);
        }
        Ok(Self {
            components: components.map(f32::to_bits),
            coordinate_space,
        })
    }

    pub(crate) fn from_committed_box(
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Self {
        Self {
            components: [bounds.x(), bounds.y(), bounds.width(), bounds.height()].map(f32::to_bits),
            coordinate_space: bounds.coordinate_space(),
        }
    }

    pub(crate) fn components(self) -> [f32; 4] {
        self.components.map(f32::from_bits)
    }

    pub(crate) const fn coordinate_space(self) -> worth_ui_host_contract::UiMountedCoordinateSpace {
        self.coordinate_space
    }
}

impl UiMotionTransitionRequest {
    pub(super) const fn with_policy(mut self, policy: crate::declaration::UiMotionPolicy) -> Self {
        self.declaration = self.declaration.with_policy(policy);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_family_transition(
        target: UiMotionTargetIdentity,
        predecessor_revision: u64,
        successor_revision: u64,
        predecessor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        predecessor_geometry: Option<UiMotionSemanticGeometry>,
        predecessor_visible: bool,
        successor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        successor_geometry: Option<UiMotionSemanticGeometry>,
        successor_visible: bool,
        declaration: super::UiMotionDeclaration,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        Self::from_family_transition_inner(
            target,
            predecessor_revision,
            successor_revision,
            predecessor_presentation,
            predecessor_geometry,
            predecessor_visible,
            successor_presentation,
            successor_geometry,
            successor_visible,
            declaration,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_rebind_transition(
        target: UiMotionTargetIdentity,
        predecessor_revision: u64,
        successor_revision: u64,
        predecessor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        predecessor_geometry: Option<UiMotionSemanticGeometry>,
        predecessor_visible: bool,
        successor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        successor_geometry: Option<UiMotionSemanticGeometry>,
        successor_visible: bool,
        declaration: super::UiMotionDeclaration,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        Self::from_family_transition_inner(
            target,
            predecessor_revision,
            successor_revision,
            predecessor_presentation,
            predecessor_geometry,
            predecessor_visible,
            successor_presentation,
            successor_geometry,
            successor_visible,
            declaration,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_family_transition_inner(
        target: UiMotionTargetIdentity,
        predecessor_revision: u64,
        successor_revision: u64,
        predecessor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        predecessor_geometry: Option<UiMotionSemanticGeometry>,
        predecessor_visible: bool,
        successor_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        successor_geometry: Option<UiMotionSemanticGeometry>,
        successor_visible: bool,
        declaration: super::UiMotionDeclaration,
        rebind: bool,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        if successor_revision <= predecessor_revision {
            return Err(UiMotionTransitionRequestDenial::RevisionDidNotAdvance);
        }
        if !rebind
            && predecessor_presentation.host_surface() != successor_presentation.host_surface()
        {
            return Err(UiMotionTransitionRequestDenial::SurfaceChanged);
        }
        if !rebind && predecessor_presentation.binding() != successor_presentation.binding() {
            return Err(UiMotionTransitionRequestDenial::BindingChangedWithoutRebind);
        }
        if predecessor_geometry
            .zip(successor_geometry)
            .is_some_and(|(predecessor, successor)| {
                predecessor.coordinate_space() != successor.coordinate_space()
            })
        {
            return Err(UiMotionTransitionRequestDenial::GeometryCoordinateSpaceChanged);
        }
        Ok(Self {
            predecessor: UiCommittedMotionPredecessorReceipt {
                geometry: predecessor_geometry,
                visible: predecessor_visible,
            },
            successor: UiPreparedMotionSuccessorReceipt {
                target,
                owner_revision: successor_revision,
                presentation: successor_presentation,
                geometry: successor_geometry,
                visible: successor_visible,
            },
            declaration,
        })
    }

    pub(in crate::runtime) const fn predecessor(self) -> UiCommittedMotionPredecessorReceipt {
        self.predecessor
    }

    pub(in crate::runtime) const fn successor(self) -> UiPreparedMotionSuccessorReceipt {
        self.successor
    }

    pub(in crate::runtime) const fn declaration(self) -> super::UiMotionDeclaration {
        self.declaration
    }

    pub(in crate::runtime) fn bind_published_successor(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        if self.successor.presentation.host_surface() != presentation.host_surface() {
            return Err(UiMotionTransitionRequestDenial::SurfaceChanged);
        }
        if self.successor.presentation.binding() != presentation.binding() {
            return Err(UiMotionTransitionRequestDenial::BindingChangedWithoutRebind);
        }
        self.successor.presentation = presentation;
        Ok(self)
    }
}

impl UiCommittedMotionPredecessorReceipt {
    pub(in crate::runtime) const fn geometry(self) -> Option<UiMotionSemanticGeometry> {
        self.geometry
    }

    pub(in crate::runtime) const fn visible(self) -> bool {
        self.visible
    }
}

impl UiPreparedMotionSuccessorReceipt {
    pub(in crate::runtime) const fn target(self) -> UiMotionTargetIdentity {
        self.target
    }

    pub(in crate::runtime) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }

    pub(in crate::runtime) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(in crate::runtime) const fn geometry(self) -> Option<UiMotionSemanticGeometry> {
        self.geometry
    }

    pub(in crate::runtime) const fn visible(self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_is_branded_only_after_finite_non_negative_validation() {
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [0.0, 0.0, -1.0, 1.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiMotionGeometryDenial::NegativeExtent)
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [f32::NAN, 0.0, 1.0, 1.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiMotionGeometryDenial::NonFinite)
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [1.0, 2.0, 3.0, 4.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            )
            .unwrap()
            .components(),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [1.0, 2.0, 3.0, 4.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            )
            .unwrap()
            .coordinate_space(),
            worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        );
    }
}
