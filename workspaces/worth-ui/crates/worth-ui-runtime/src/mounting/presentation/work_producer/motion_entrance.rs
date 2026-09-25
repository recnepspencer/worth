//! Lower the proposal's first sample against concrete candidate Portal commands.
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial as Denial, UiMountedPresentationSampleChange,
    UiMountedPresentationTransform,
};

impl super::UiMountedPresentationState {
    pub(in crate::mounting::presentation) fn prepare_motion_entrance(
        &mut self,
        entrance: crate::runtime::motion::UiPreparedMotionEntrance,
    ) -> Result<Vec<UiMountedPresentationSampleChange>, Denial> {
        if entrance.frame() != self.frame {
            return Err(Denial::StalePredecessor);
        }
        if entrance.target().semantic_surface() != self.requirement.semantic_surface() {
            return Ok(Vec::new());
        }
        let group = self
            .portal_motion_group(entrance.target())
            .ok_or(Denial::MalformedProjection)?;
        let transform = match entrance.geometry() {
            (Some(source), Some(initial)) => Some(
                UiMountedPresentationTransform::from_runtime_sampling(
                    source.canonical_box(),
                    initial.canonical_box(),
                )
                .map_err(|_| Denial::MalformedProjection)?,
            ),
            (None, None) => None,
            _ => return Err(Denial::MalformedProjection),
        };
        let changes = group
            .commands()
            .map(|identity| {
                UiMountedPresentationSampleChange::from_runtime_sampling(
                    identity,
                    transform,
                    super::super::compose_opacity(self.appearance_opacity_for_command(identity), 0),
                )
            })
            .collect::<Vec<_>>();
        if changes.is_empty() {
            return Err(Denial::MalformedProjection);
        }
        self.retain_entrance_acceptance(entrance, &changes)?;
        Ok(changes)
    }
}
