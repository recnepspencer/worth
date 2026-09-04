use super::WorthUiActiveApplicationSession;
use crate::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameIdentity, UiMountedGraphNodeHandle,
    UiMountedIdentityDenial, UiMountedIdentityView, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiMountedProjectionAudience, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration, UiSurfaceBindingIdentityView, UiSurfaceBindingProfile,
};

/// SUPPORT AUTHORITY for constructing hostile mounted-identity worlds.
pub trait WorthUiMountedIdentityCertificationExt {
    fn create_semantic_surface(
        &mut self,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial>;

    fn create_semantic_surface_for(
        &mut self,
        audience: UiMountedProjectionAudience,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial>;

    fn mounted_graph_node(
        &self,
        graph_node_identity: crate::graph::UiGraphNodeIdentity,
    ) -> Result<UiMountedGraphNodeHandle, UiMountedIdentityDenial>;

    fn register_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial>;

    fn deregister_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial>;

    fn rebind_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial>;

    fn recover_indeterminate_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial>;

    fn mount_instance(
        &mut self,
        node: UiMountedGraphNodeHandle,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiMountedInstanceIdentity, UiMountedIdentityDenial>;

    fn unmount_instance(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<(), UiMountedIdentityDenial>;

    fn reorder_mounted_instances(
        &mut self,
        order: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial>;

    fn mounted_instances_for(
        &self,
        node: UiMountedGraphNodeHandle,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedIdentityDenial>;

    fn advance_mounted_identity_frame(
        &mut self,
    ) -> Result<UiMountedFrameIdentity, UiMountedIdentityDenial>;

    fn validate_current_surface_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(), UiMountedIdentityDenial>;

    fn validate_current_mounted_frame(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<(), UiMountedIdentityDenial>;

    fn validate_current_mounted_node_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> Result<(), UiMountedIdentityDenial>;

    fn inspect_mounted_identity(&self) -> UiMountedIdentityView;
}

impl WorthUiActiveApplicationSession {
    pub(crate) fn create_semantic_surface(
        &mut self,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        let surface = self.mounted.create_semantic_surface()?;
        self.materialize_initial_appearance_theme_binding(surface)?;
        Ok(surface)
    }

    pub(crate) fn create_semantic_surface_for(
        &mut self,
        audience: UiMountedProjectionAudience,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        let surface = self.mounted.create_semantic_surface_for(audience)?;
        self.materialize_initial_appearance_theme_binding(surface)?;
        Ok(surface)
    }

    pub(super) fn materialize_initial_appearance_theme_binding(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        let Some(admission) = self.appearance_theme_admission.as_ref().cloned() else {
            return Ok(());
        };
        self.presentation
            .materialize_initial_appearance_theme_binding(admission.materialize(surface))
            .map_err(|_| UiMountedIdentityDenial::SurfaceAlreadyBound)
    }

    pub(crate) fn mounted_graph_node(
        &self,
        graph_node_identity: crate::graph::UiGraphNodeIdentity,
    ) -> Result<UiMountedGraphNodeHandle, UiMountedIdentityDenial> {
        self.mounted
            .graph_node_handle(self.application.graph(), graph_node_identity)
    }

    pub(crate) fn register_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        self.mounted
            .register_host_surface(&self.host_session, semantic_surface, mode, profile)
    }

    pub(crate) fn deregister_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        let semantic_surface = self
            .mounted
            .deregister_host_surface(&self.host_session, binding)?;
        self.intent_confirmation.cancel_binding(
            binding,
            crate::runtime::intent::UiIntentConfirmationCancellationReason::SurfaceRebound,
        );
        self.intent_admission
            .cancel_binding(&mut self.intent_execution, binding);
        let previous_input = self.interaction.active_input_binding();
        self.interaction.cancel_binding(
            binding,
            crate::runtime::interaction::UiInteractionLifecycleStopReason::SurfaceRebound,
        );
        self.clear_displaced_input_recipient(previous_input);
        Ok(semantic_surface)
    }

    pub(crate) fn rebind_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        self.rebind_host_surface_with_interaction_receipt(binding, mode, profile)
            .map(|receipt| receipt.binding())
            .map_err(|denial| denial.mounted_denial())
    }

    pub(crate) fn recover_indeterminate_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.mounted
            .recover_indeterminate_host_surface(&self.host_session, semantic_surface)
    }

    pub(crate) fn mount_instance(
        &mut self,
        node: UiMountedGraphNodeHandle,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiMountedInstanceIdentity, UiMountedIdentityDenial> {
        let mounted = self
            .mounted
            .mount_instance(self.application.graph(), node, surface)?;
        let basis = self
            .mounted
            .current_mounted_identity_basis(mounted)
            .expect("a newly mounted identity has its exact mounted basis");
        if self.scroll.is_installed() {
            let incarnation =
                crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                    basis.mount_incarnation(),
                );
            self.application.install_scroll_ownership(
                self.scroll
                    .as_mut()
                    .expect("Scroll installation was checked above"),
                mounted,
                incarnation,
                &basis,
            );
        }
        Ok(mounted)
    }

    pub(crate) fn unmount_instance(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.unmount_instance_with_interaction_receipt(identity)?;
        Ok(())
    }

    pub(crate) fn reorder_mounted_instances(
        &mut self,
        order: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        self.mounted.reorder_mounted_instances(order)
    }

    pub(crate) fn mounted_instances_for(
        &self,
        node: UiMountedGraphNodeHandle,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedIdentityDenial> {
        self.mounted.mounted_instances_for(node)
    }

    pub(crate) fn advance_mounted_identity_frame(
        &mut self,
    ) -> Result<UiMountedFrameIdentity, UiMountedIdentityDenial> {
        self.mounted.advance_frame()
    }

    pub(crate) fn validate_current_surface_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.mounted.validate_binding(binding)
    }

    pub(crate) fn validate_current_mounted_frame(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.mounted.validate_current_frame(frame)
    }

    pub(crate) fn validate_current_mounted_node_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.mounted.validate_current_receipt(instance, receipt)
    }

    pub(crate) fn inspect_mounted_identity(&self) -> UiMountedIdentityView {
        self.mounted.view()
    }
}

impl WorthUiMountedIdentityCertificationExt for WorthUiActiveApplicationSession {
    fn create_semantic_surface(
        &mut self,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::create_semantic_surface(self)
    }

    fn create_semantic_surface_for(
        &mut self,
        audience: UiMountedProjectionAudience,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::create_semantic_surface_for(self, audience)
    }

    fn mounted_graph_node(
        &self,
        graph_node_identity: crate::graph::UiGraphNodeIdentity,
    ) -> Result<UiMountedGraphNodeHandle, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::mounted_graph_node(self, graph_node_identity)
    }

    fn register_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::register_host_surface(
            self,
            semantic_surface,
            mode,
            profile,
        )
    }

    fn deregister_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::deregister_host_surface(self, binding)
    }

    fn rebind_host_surface(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::rebind_host_surface(self, binding, mode, profile)
    }

    fn recover_indeterminate_host_surface(
        &mut self,
        semantic_surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::recover_indeterminate_host_surface(self, semantic_surface)
    }

    fn mount_instance(
        &mut self,
        node: UiMountedGraphNodeHandle,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiMountedInstanceIdentity, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::mount_instance(self, node, surface)
    }

    fn unmount_instance(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::unmount_instance(self, identity)
    }

    fn reorder_mounted_instances(
        &mut self,
        order: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::reorder_mounted_instances(self, order)
    }

    fn mounted_instances_for(
        &self,
        node: UiMountedGraphNodeHandle,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::mounted_instances_for(self, node)
    }

    fn advance_mounted_identity_frame(
        &mut self,
    ) -> Result<UiMountedFrameIdentity, UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::advance_mounted_identity_frame(self)
    }

    fn validate_current_surface_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::validate_current_surface_binding(self, binding)
    }

    fn validate_current_mounted_frame(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::validate_current_mounted_frame(self, frame)
    }

    fn validate_current_mounted_node_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        WorthUiActiveApplicationSession::validate_current_mounted_node_receipt(
            self, instance, receipt,
        )
    }

    fn inspect_mounted_identity(&self) -> UiMountedIdentityView {
        WorthUiActiveApplicationSession::inspect_mounted_identity(self)
    }
}
