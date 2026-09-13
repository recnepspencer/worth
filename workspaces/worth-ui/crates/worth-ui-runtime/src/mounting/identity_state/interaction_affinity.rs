use worth_ui_host_contract::{
    UiMountedHitTestMechanic, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use super::UiMountedIdentityState;

/// Why a historically presented hit row can no longer name a live target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiCurrentHitTargetAffinityDenial {
    PresentationNotCurrent,
    SurfaceNoLongerBound,
    BindingNoLongerCurrent,
    MountedInstanceNoLongerCurrent,
    MountedSurfaceAffinityChanged,
}

/// Mounting-owned admission that one exact presented row still names the same
/// live mounted incarnation. The row cannot be substituted after admission.
pub(crate) struct UiCurrentHitTarget {
    row: UiMountedHitTestMechanic,
}

pub(crate) struct UiCurrentInteractionAffinity {
    graph_node: crate::graph::UiGraphNodeIdentity,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedIncarnationAffinityInput {
    pub(crate) surface: UiSemanticSurfaceIdentity,
    pub(crate) binding: UiSurfaceBindingGeneration,
    pub(crate) mounted_instance: UiMountedInstanceIdentity,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedInteractionAffinityInput {
    pub(crate) surface: UiSemanticSurfaceIdentity,
    pub(crate) binding: UiSurfaceBindingGeneration,
    pub(crate) mounted_instance: UiMountedInstanceIdentity,
    pub(crate) node_receipt: UiMountedNodeReceiptIdentity,
}

impl UiMountedIdentityState {
    pub(crate) fn admit_current_hit_target(
        &self,
        row: UiMountedHitTestMechanic,
    ) -> Result<UiCurrentHitTarget, UiCurrentHitTargetAffinityDenial> {
        let binding = self
            .bindings
            .get(&row.surface())
            .ok_or(UiCurrentHitTargetAffinityDenial::SurfaceNoLongerBound)?;
        if binding.view.binding_generation() != row.binding() {
            return Err(UiCurrentHitTargetAffinityDenial::BindingNoLongerCurrent);
        }
        let instance = self
            .instances
            .get(&row.mounted_instance())
            .ok_or(UiCurrentHitTargetAffinityDenial::MountedInstanceNoLongerCurrent)?;
        if instance.basis.semantic_surface_identity() != row.surface() {
            return Err(UiCurrentHitTargetAffinityDenial::MountedSurfaceAffinityChanged);
        }
        Ok(UiCurrentHitTarget { row })
    }

    pub(crate) fn admit_current_mounted_incarnation_affinity(
        &self,
        input: UiMountedIncarnationAffinityInput,
    ) -> Result<UiCurrentInteractionAffinity, UiCurrentHitTargetAffinityDenial> {
        let binding = self
            .bindings
            .get(&input.surface)
            .ok_or(UiCurrentHitTargetAffinityDenial::SurfaceNoLongerBound)?;
        if binding.view.binding_generation() != input.binding {
            return Err(UiCurrentHitTargetAffinityDenial::BindingNoLongerCurrent);
        }
        let instance = self
            .instances
            .get(&input.mounted_instance)
            .ok_or(UiCurrentHitTargetAffinityDenial::MountedInstanceNoLongerCurrent)?;
        if instance.basis.semantic_surface_identity() != input.surface {
            return Err(UiCurrentHitTargetAffinityDenial::MountedSurfaceAffinityChanged);
        }
        Ok(UiCurrentInteractionAffinity {
            graph_node: instance.basis.graph_node_identity(),
        })
    }
}

impl UiCurrentHitTarget {
    pub(crate) const fn row(&self) -> UiMountedHitTestMechanic {
        self.row
    }
}

impl UiCurrentInteractionAffinity {
    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }
}
