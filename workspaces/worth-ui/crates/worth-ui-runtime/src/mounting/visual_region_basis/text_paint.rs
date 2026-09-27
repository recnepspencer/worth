use super::{UiMountedUnsupportedPaintBasis, UiMountedVisualRegionBasis};
use crate::mounting::projection::UiMountedAppearanceClip as Clip;

impl UiMountedVisualRegionBasis {
    pub(in crate::mounting) fn with_text_clips(
        mut self,
        clips: std::collections::BTreeMap<worth_ui_host_contract::UiMountedInstanceIdentity, Clip>,
    ) -> Self {
        self.text_clips = std::rc::Rc::new(clips);
        self
    }
    pub(crate) fn unsupported_paint(&self) -> Box<[UiMountedUnsupportedPaintBasis]> {
        #[cfg(test)]
        if let Some(materialized) = &self.materialized {
            return materialized.unsupported_paint.iter().copied().collect();
        }
        self.semantic_text
            .visual_mechanics()
            .filter(|row| self.binding.is_none_or(|binding| row.binding() == binding))
            .filter_map(|row| {
                // The mechanic source holds each row where layout put it.
                let presented = self
                    .placement_of(row.mounted_instance())
                    .present(crate::mounting::UiLaidOut::from_layout(row.clone()))
                    .expect("validated Portal-relative text remains canonical")?
                    .into_shown();
                let presented = match self.text_clips.get(&presented.mounted_instance()) {
                    None | Some(Clip::Unclipped) => Some(presented),
                    Some(Clip::Suppressed) => None,
                    Some(Clip::Ancestor(clip)) => presented
                        .clipped_to_appearance_ancestor(*clip)
                        .expect("prepared ancestor text clipping remains canonical"),
                    Some(Clip::Unresolved(_)) => panic!("presented text has completed geometry"),
                }?;
                Some(UiMountedUnsupportedPaintBasis {
                    node_receipt: self
                        .receipts
                        .as_ref()
                        .and_then(|receipts| receipts.receipt_for(presented.mounted_instance()))
                        .unwrap_or_else(|| presented.node_receipt()),
                    bounds: presented.bounds(),
                    clip: presented.clip_bounds(),
                    semantic_order: presented.layer_semantic_order(),
                    source_digest: presented.semantic_digest(),
                })
            })
            .collect()
    }
}
