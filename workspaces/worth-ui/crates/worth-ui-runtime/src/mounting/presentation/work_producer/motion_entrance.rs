//! Lower the proposal's first sample against concrete candidate Portal commands.
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial as Denial, UiMountedPresentationSampleChange,
    UiMountedPresentationTransform,
};

use super::command_motion_layers::{
    UiCommandMotionLayer, UiCommandMotionLayerKind, UiCommandMotionLayers,
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
        // The entrance's first sample moves the Portal's content and the bars
        // it carries, over whatever else the host already shows them through.
        let mut entered = UiCommandMotionLayers::default();
        entered
            .sample(
                UiCommandMotionLayerKind::Portal,
                UiCommandMotionLayer::moved(transform, 0),
            )
            .map_err(|_| Denial::MalformedProjection)?;
        let changes = group
            .commands()
            .chain(self.scroll_motion_groups.portal_chrome(entrance.target()))
            .map(|identity| {
                let layers = entered.over(self.accepted_motion_layers(identity));
                self.resting_opacity(identity)
                    .and_then(|resting| layers.change(identity, resting))
                    .map(|change| (change, layers))
                    .map_err(|_| Denial::MalformedProjection)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if changes.is_empty() {
            return Err(Denial::MalformedProjection);
        }
        self.retain_entrance_acceptance(entrance, &changes)?;
        Ok(changes.into_iter().map(|(change, _)| change).collect())
    }
}
