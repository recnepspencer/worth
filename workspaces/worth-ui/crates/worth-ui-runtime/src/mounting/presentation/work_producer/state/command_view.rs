use super::{
    PresentationOrderKey, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiMountedPaintOrderIdentity, UiMountedPresentationState,
};

impl UiMountedPresentationState {
    pub(in crate::mounting::presentation) fn frame(
        &self,
    ) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub(in crate::mounting::presentation::work_producer) fn command_option(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<&UiMountedPaintCommand> {
        self.commands_by_instance
            .get(&identity.mounted_instance())?
            .get(identity)
    }

    pub(in crate::mounting::presentation::work_producer) fn command(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> &UiMountedPaintCommand {
        self.command_option(identity)
            .expect("paint order names a retained command")
    }

    pub(in crate::mounting::presentation) fn command_identities_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.commands_by_instance
            .get(&instance)
            .into_iter()
            .flat_map(super::super::command_bundle::UiMountedPresentationCommandBundle::iter)
            .map(UiMountedPaintCommand::identity)
    }

    pub(in crate::mounting::presentation) fn accepted_appearance_motion_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> impl Iterator<
        Item = (
            UiMountedPaintCommandIdentity,
            bool,
            bool,
            Option<super::super::super::motion_sampling::UiPresentationMotionSampleReceipt>,
        ),
    > + '_ {
        self.commands_by_instance
            .get(&instance)
            .into_iter()
            .flat_map(|bundle| bundle.appearance_motion())
            .map(|(command, accepted)| {
                let portal = matches!(command, UiMountedPaintCommand::PortalOverlay { .. });
                let instance_composable =
                    !matches!(command, UiMountedPaintCommand::SemanticText { .. });
                (command.identity(), portal, instance_composable, accepted)
            })
    }

    pub(in crate::mounting::presentation::work_producer) fn reconstruction_appearance_motion(
        &self,
    ) -> impl Iterator<
        Item = (
            UiMountedPaintCommandIdentity,
            Option<super::super::super::motion_sampling::UiPresentationMotionSampleReceipt>,
        ),
    > + '_ {
        self.command_order.iter().filter_map(|(_, identity)| {
            self.motion_for_command(*identity)
                .map(|accepted| (*identity, accepted))
        })
    }

    pub(in crate::mounting::presentation) fn portal_motion_group(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Option<super::super::portal_motion_groups::UiMountedPortalMotionGroupView<'_>> {
        self.portal_motion_groups.group(target)
    }

    pub(in crate::mounting::presentation::work_producer) fn order_key(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<PresentationOrderKey> {
        let instance = identity.mounted_instance();
        let position = self.instance_order.position(instance)?;
        let local = self
            .commands_by_instance
            .get(&instance)?
            .position(identity)?;
        let command = self.commands_by_instance.get(&instance)?.get(identity)?;
        Some((command.layer_semantic_order(), position, local))
    }

    pub(in crate::mounting::presentation::work_producer) fn previous_order(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<UiMountedPaintOrderIdentity> {
        let key = self.order_key(identity)?;
        self.command_order
            .predecessor(&key)
            .map(|(_, command)| UiMountedPaintOrderIdentity::for_command(*command))
    }

    pub(in crate::mounting::presentation::work_producer) fn order(
        &self,
    ) -> Vec<UiMountedPaintOrderIdentity> {
        self.command_order
            .iter()
            .map(|(_, command)| UiMountedPaintOrderIdentity::for_command(*command))
            .collect()
    }

    pub(in crate::mounting::presentation::work_producer) fn commands(
        &self,
    ) -> Vec<UiMountedPaintCommand> {
        self.command_order
            .iter()
            .filter_map(|(_, identity)| self.command_option(*identity).cloned())
            .collect()
    }

    pub(in crate::mounting::presentation::work_producer) fn source_instance_count(&self) -> usize {
        self.commands_by_instance.len()
    }
}
