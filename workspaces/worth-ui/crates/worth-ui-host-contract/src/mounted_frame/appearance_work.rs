use crate::{
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFrameProjection,
};

/// Appearance mechanics admitted for one surface in one mounted presentation
/// attempt, and the Motion samples the attempt reissues there. A frame that
/// changes no appearance on the surface can still owe the host samples: what
/// the host shows an unchanged command through ends with the frame.
pub struct UiMountedAppearancePresentationWork {
    frame: crate::UiMountedFrameIdentity,
    presentation: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
    fragments: Box<[UiUnpublishedAppearanceFragment]>,
    sample_overrides: Box<[crate::UiMountedPresentationSampleChange]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedAppearancePresentationWorkDenial {
    FrameMismatch,
    PresentationMismatch,
    SurfaceBindingMismatch,
}

impl UiMountedAppearancePresentationWork {
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        source: &UiUnpublishedAppearanceFrameProjection,
        frame: crate::UiMountedFrameIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        requirement: UiMountedSurfaceBindingRequirement,
        sample_overrides: impl IntoIterator<Item = crate::UiMountedPresentationSampleChange>,
    ) -> Result<Option<Self>, UiMountedAppearancePresentationWorkDenial> {
        if source.frame() != frame {
            return Err(UiMountedAppearancePresentationWorkDenial::FrameMismatch);
        }
        if source.presentation() != presentation {
            return Err(UiMountedAppearancePresentationWorkDenial::PresentationMismatch);
        }
        let fragments = source
            .fragments()
            .iter()
            .filter(|fragment| {
                fragment.surface_binding().semantic_surface() == requirement.semantic_surface()
            })
            .map(|fragment| {
                if fragment.surface_binding() != requirement {
                    Err(UiMountedAppearancePresentationWorkDenial::SurfaceBindingMismatch)
                } else {
                    Ok(fragment.clone())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::admitted(
            frame,
            presentation,
            requirement,
            fragments,
            sample_overrides,
        )
    }

    /// The samples a frame that changes no appearance reissues on one
    /// surface; no work when it reissues none.
    #[doc(hidden)]
    pub fn from_runtime_sample_overrides(
        frame: crate::UiMountedFrameIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        requirement: UiMountedSurfaceBindingRequirement,
        sample_overrides: impl IntoIterator<Item = crate::UiMountedPresentationSampleChange>,
    ) -> Result<Option<Self>, UiMountedAppearancePresentationWorkDenial> {
        Self::admitted(
            frame,
            presentation,
            requirement,
            Vec::new(),
            sample_overrides,
        )
    }

    /// Each command's sample is reissued at most once; work with neither
    /// fragments nor samples is no work.
    fn admitted(
        frame: crate::UiMountedFrameIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        requirement: UiMountedSurfaceBindingRequirement,
        fragments: Vec<UiUnpublishedAppearanceFragment>,
        sample_overrides: impl IntoIterator<Item = crate::UiMountedPresentationSampleChange>,
    ) -> Result<Option<Self>, UiMountedAppearancePresentationWorkDenial> {
        let sample_overrides = sample_overrides.into_iter().collect::<Vec<_>>();
        if fragments.is_empty() && sample_overrides.is_empty() {
            return Ok(None);
        }
        let unique = sample_overrides
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>();
        if unique.len() != sample_overrides.len() {
            return Err(UiMountedAppearancePresentationWorkDenial::PresentationMismatch);
        }
        Ok(Some(Self {
            frame,
            presentation,
            requirement,
            fragments: fragments.into_boxed_slice(),
            sample_overrides: sample_overrides.into_boxed_slice(),
        }))
    }

    pub const fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.frame
    }

    pub const fn presentation(&self) -> UiMountedPresentationAttemptIdentity {
        self.presentation
    }

    pub const fn requirement(&self) -> UiMountedSurfaceBindingRequirement {
        self.requirement
    }

    pub fn fragments(&self) -> &[UiUnpublishedAppearanceFragment] {
        &self.fragments
    }

    /// Only the surface overlay fragment carries the issued stack update.
    /// Its absence preserves retained order on incremental presentation; an
    /// explicit empty update removes that order.
    pub fn overlay_order_update(&self) -> Option<&crate::UiMountedOverlayOrderMechanic> {
        self.fragments.iter().find_map(|fragment| {
            matches!(
                fragment.identity(),
                crate::UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(_)
            )
            .then(|| fragment.work().successor().overlay_order())
        })
    }

    pub fn sample_overrides(&self) -> &[crate::UiMountedPresentationSampleChange] {
        &self.sample_overrides
    }
}

#[cfg(test)]
#[path = "appearance_work_tests.rs"]
mod tests;
