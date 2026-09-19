use worth_ui_host_contract::{UiMountedPaintCommand, UiMountedPresentationUnchanged};

use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial};

pub(in crate::native) struct UiNativeRetainedUnchangedUndo {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    content: worth_ui_host_contract::UiMountedContentGeneration,
    regions: super::super::retained_regions::UiNativeRetainedRegions,
    appearance: Vec<crate::native::presentation::appearance::UiNativeAppearanceCommandUndo>,
    appearance_samples: Vec<super::sample_transaction::UiNativeAppearanceSampleUndo>,
}

impl UiNativeRetainedDrawList {
    pub(crate) fn apply_unchanged(
        &mut self,
        unchanged: &UiMountedPresentationUnchanged,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let affinity = unchanged.affinity();
        if affinity.predecessor() != Some(self.frame)
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
            || affinity.baseline() != self.baseline
        {
            return Err(UiNativeRetainedDrawListDenial::AffinityMismatch);
        }
        self.regions
            .rebind_receipt_affinity(affinity.receipt_affinity());
        self.frame = affinity.successor();
        self.content = affinity.content();
        Ok(())
    }

    pub(in crate::native) fn stage_unchanged(
        &mut self,
        unchanged: &UiMountedPresentationUnchanged,
    ) -> Result<UiNativeRetainedUnchangedUndo, UiNativeRetainedDrawListDenial> {
        let undo = UiNativeRetainedUnchangedUndo {
            frame: self.frame,
            content: self.content,
            regions: self.regions.clone(),
            appearance: Vec::new(),
            appearance_samples: Vec::new(),
        };
        self.apply_unchanged(unchanged)?;
        Ok(undo)
    }

    pub(in crate::native) fn retain_unchanged_appearance(
        undo: &mut UiNativeRetainedUnchangedUndo,
        command: crate::native::presentation::appearance::UiNativeAppearanceCommandUndo,
    ) {
        undo.appearance.push(command);
    }

    pub(in crate::native) fn retain_unchanged_appearance_samples(
        undo: &mut UiNativeRetainedUnchangedUndo,
        samples: super::sample_transaction::UiNativeAppearanceSampleUndo,
    ) {
        undo.appearance_samples.push(samples);
    }

    pub(in crate::native) fn rollback_unchanged(
        &mut self,
        undo: UiNativeRetainedUnchangedUndo,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        for samples in undo.appearance_samples.into_iter().rev() {
            self.rollback_appearance_sample_overrides(samples)?;
        }
        if !undo.appearance.is_empty() {
            self.rollback_appearance_commands(undo.appearance.into_iter().rev())?;
        }
        self.frame = undo.frame;
        self.content = undo.content;
        self.regions = undo.regions;
        Ok(())
    }

    pub(in crate::native::presentation) fn reconstruction_commands(
        &self,
    ) -> Result<Vec<&UiMountedPaintCommand>, UiNativeRetainedDrawListDenial> {
        self.order
            .ordered()
            .map(|identity| {
                self.commands
                    .get(&identity.command())
                    .ok_or(UiNativeRetainedDrawListDenial::OrderMismatch)
            })
            .collect()
    }
}
