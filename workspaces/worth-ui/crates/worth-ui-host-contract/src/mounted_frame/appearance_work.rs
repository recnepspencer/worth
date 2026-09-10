use crate::{
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFrameProjection,
};

/// Appearance mechanics admitted for one surface in one mounted presentation attempt.
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
        if fragments.is_empty() {
            return Ok(None);
        }
        let sample_overrides = sample_overrides.into_iter().collect::<Vec<_>>();
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

    pub fn sample_overrides(&self) -> &[crate::UiMountedPresentationSampleChange] {
        &self.sample_overrides
    }
}
