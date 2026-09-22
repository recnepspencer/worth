//! Ordinary-frame base glyph attribution, shared by same-frame samples.
//! These rows describe admitted glyph runs, not sampled display positions.
use super::{text, UiNativeRetainedDrawList};
use crate::native::text_atlas::{UiNativeTextAtlas, UiNativeTextAtlasContentRevision};
use crate::native::UiNativeGlyphObservation;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct UiNativeBaseGlyphObservations {
    pub(super) intrinsic: Arc<[UiNativeGlyphObservation]>,
    pub(super) alpha: Arc<[UiNativeGlyphObservation]>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Basis {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    extent: [u32; 2],
    atlas: UiNativeTextAtlasContentRevision,
}

#[derive(Default)]
pub(super) struct UiNativeGlyphObservationCache {
    value: RefCell<Option<(Basis, UiNativeBaseGlyphObservations)>>,
    #[cfg(test)]
    builds: std::cell::Cell<usize>,
}

impl UiNativeGlyphObservationCache {
    pub(super) fn observe(
        &self,
        retained: &UiNativeRetainedDrawList,
        atlas: &UiNativeTextAtlas,
        extent: [u32; 2],
    ) -> UiNativeBaseGlyphObservations {
        let basis = Basis {
            frame: retained.frame(),
            extent,
            atlas: atlas.committed_content_revision(),
        };
        if let Some((_, observations)) = self
            .value
            .borrow()
            .as_ref()
            .filter(|(key, _)| *key == basis)
        {
            return observations.clone();
        }
        let observations = collect(retained, atlas, extent);
        #[cfg(test)]
        self.builds.set(self.builds.get() + 1);
        *self.value.borrow_mut() = Some((basis, observations.clone()));
        observations
    }

    #[cfg(test)]
    pub(super) fn build_count(&self) -> usize {
        self.builds.get()
    }
}

pub(super) fn observe(
    retained: &UiNativeRetainedDrawList,
    atlas: &UiNativeTextAtlas,
    extent: [u32; 2],
) -> UiNativeBaseGlyphObservations {
    retained.glyph_observation.observe(retained, atlas, extent)
}

fn collect(
    retained: &UiNativeRetainedDrawList,
    atlas: &UiNativeTextAtlas,
    extent: [u32; 2],
) -> UiNativeBaseGlyphObservations {
    let runs = retained.all_glyph_runs();
    let commands = text::plan_glyph_commands(&runs, atlas, extent)
        .expect("retained qualified glyph runs have atlas entries");
    let mut intrinsic = Vec::new();
    let mut alpha = Vec::new();
    for command in commands.iter().copied() {
        let destination = if text::source_is_intrinsic_color(command) {
            &mut intrinsic
        } else {
            &mut alpha
        };
        destination.push(UiNativeGlyphObservation::from_native_command(command));
    }
    for observations in [&mut intrinsic, &mut alpha] {
        observations.sort_by_key(|observation| {
            (
                observation.original_range(),
                observation.glyph_id(),
                observation.target_bounds(),
            )
        });
    }
    UiNativeBaseGlyphObservations {
        intrinsic: intrinsic.into(),
        alpha: alpha.into(),
    }
}
