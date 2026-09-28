use super::active_application_session::UiScrollSettlementScope;
use super::WorthUiActiveApplicationSession;
use crate::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedIdentityDenial, UiMountedInstanceIdentity,
    UiSurfaceBindingGeneration, UiSurfaceBindingIdentityView, UiSurfaceBindingProfile,
};
use crate::runtime::interaction::{
    UiInteractionLifecycleSettlementReceipt, UiInteractionLifecycleStopReason,
};

/// Successful surface rebind plus the exact gestures retired by that boundary.
#[derive(Debug, Eq, PartialEq)]
pub struct UiSurfaceRebindInteractionReceipt {
    binding: UiSurfaceBindingIdentityView,
    interaction: UiInteractionLifecycleSettlementReceipt,
}

/// A rebind denial that preserves settlement if deregistration already occurred.
#[derive(Debug, Eq, PartialEq)]
pub enum UiSurfaceRebindInteractionDenial {
    BeforeMutation(UiMountedIdentityDenial),
    AfterInteractionSettlement {
        denial: UiMountedIdentityDenial,
        interaction: Box<UiInteractionLifecycleSettlementReceipt>,
    },
}

/// SUPPORT AUTHORITY for observing interaction settlement at mounted mutations.
pub trait WorthUiMountedInteractionLifecycleCertificationExt {
    fn unmount_instance_with_interaction_receipt(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<UiInteractionLifecycleSettlementReceipt, UiMountedIdentityDenial>;

    fn rebind_host_surface_with_interaction_receipt(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceRebindInteractionReceipt, UiSurfaceRebindInteractionDenial>;
}

impl WorthUiActiveApplicationSession {
    pub(crate) fn unmount_instance_with_interaction_receipt(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<UiInteractionLifecycleSettlementReceipt, UiMountedIdentityDenial> {
        let service_basis = self.mounted.current_mounted_identity_basis(identity);
        self.mounted.unmount_instance(identity)?;
        // The occurrence is gone, so anything still walking its content toward
        // a wheel target is walking content that no longer exists. This runs
        // before Scroll retires the occurrence below, because retiring it is
        // what removes the ownership chain that says which settles were its.
        self.settle_live_scroll_motion(
            UiScrollSettlementScope::MountedOccurrence(identity),
            crate::runtime::motion::UiMotionTerminalCause::SubjectUnmounted,
        );
        if self.scroll.is_installed() {
            self.scroll
                .as_mut()
                .expect("Scroll installation was checked above")
                .retire_mounted_instance(identity);
        }
        if self.selection.is_installed() {
            if let Some(basis) = service_basis {
                self.selection
                    .as_mut()
                    .expect("Selection installation was checked above")
                    .retire_mounted_owner(
                    basis.semantic_surface_identity(),
                    basis.graph_node_identity(),
                    crate::runtime::selection::UiSelectionOwnerIncarnation::from_mount_incarnation(
                        basis.mount_incarnation(),
                    ),
                );
            }
        }
        self.intent_confirmation.cancel_instance(
            identity,
            crate::runtime::intent::UiIntentConfirmationCancellationReason::MountedInstanceRemoved,
        );
        self.intent_application_facts
            .retire_validation_appearance_instance(identity);
        self.intent_admission
            .cancel_instance(&mut self.intent_execution, identity);
        let previous_input = self.interaction.active_input_binding();
        let settlement = self.interaction.cancel_instance(
            identity,
            UiInteractionLifecycleStopReason::MountedInstanceRemoved,
        );
        self.clear_displaced_input_recipient(previous_input);
        Ok(settlement)
    }

    pub(crate) fn rebind_host_surface_with_interaction_receipt(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceRebindInteractionReceipt, UiSurfaceRebindInteractionDenial> {
        if let Some(surface) = self.mounted.current_surface_for_binding(binding) {
            self.settle_owed_scroll_sample_before_ending(surface);
        }
        let (semantic_surface, host_surface) = self
            .mounted
            .deregister_host_surface_for_rebind(&self.host_session, binding)
            .map_err(UiSurfaceRebindInteractionDenial::BeforeMutation)?;
        // The binding whose rectangles this surface's content was moving
        // through is gone. A settle that outlived it would resume against
        // geometry from a presentation nobody is showing any more.
        self.settle_live_scroll_motion(
            UiScrollSettlementScope::SemanticSurface(semantic_surface),
            crate::runtime::motion::UiMotionTerminalCause::ReboundAway,
        );
        let previous_input = self.interaction.active_input_binding();
        let interaction = self
            .interaction
            .cancel_binding(binding, UiInteractionLifecycleStopReason::SurfaceRebound);
        if let Some(snapshot) = self.pointer_affordance_snapshot.as_mut() {
            snapshot.invalidate_surface(semantic_surface);
        }
        self.clear_displaced_input_recipient(previous_input);
        self.intent_confirmation.cancel_binding(
            binding,
            crate::runtime::intent::UiIntentConfirmationCancellationReason::SurfaceRebound,
        );
        match self.mounted.register_rebound_host_surface(
            &self.host_session,
            binding,
            semantic_surface,
            host_surface,
            mode,
            profile,
        ) {
            Ok(rebound) => {
                self.intent_admission.rebind_surface(
                    &mut self.intent_execution,
                    binding,
                    rebound.binding_generation(),
                );
                Ok(UiSurfaceRebindInteractionReceipt {
                    binding: rebound,
                    interaction,
                })
            }
            Err(denial) => {
                if let Some(scroll) = self.scroll.as_mut() {
                    scroll.retire_surface_direct_successions(semantic_surface);
                }
                self.intent_admission
                    .cancel_binding(&mut self.intent_execution, binding);
                Err(
                    UiSurfaceRebindInteractionDenial::AfterInteractionSettlement {
                        denial,
                        interaction: Box::new(interaction),
                    },
                )
            }
        }
    }
}

impl WorthUiMountedInteractionLifecycleCertificationExt for WorthUiActiveApplicationSession {
    fn unmount_instance_with_interaction_receipt(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<UiInteractionLifecycleSettlementReceipt, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::unmount_instance_with_interaction_receipt(self, identity)
    }

    fn rebind_host_surface_with_interaction_receipt(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceRebindInteractionReceipt, UiSurfaceRebindInteractionDenial> {
        WorthUiActiveApplicationSession::rebind_host_surface_with_interaction_receipt(
            self, binding, mode, profile,
        )
    }
}

impl UiSurfaceRebindInteractionReceipt {
    pub const fn binding(&self) -> UiSurfaceBindingIdentityView {
        self.binding
    }

    pub const fn interaction(&self) -> &UiInteractionLifecycleSettlementReceipt {
        &self.interaction
    }
}

impl UiSurfaceRebindInteractionDenial {
    pub const fn mounted_denial(&self) -> UiMountedIdentityDenial {
        match self {
            Self::BeforeMutation(denial) | Self::AfterInteractionSettlement { denial, .. } => {
                *denial
            }
        }
    }
}
