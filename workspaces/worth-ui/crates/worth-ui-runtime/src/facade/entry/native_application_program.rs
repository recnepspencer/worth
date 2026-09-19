const MAXIMUM_FRAMES: usize = 32;
const MAXIMUM_CHANGES_PER_FRAME: usize = 4_096;

#[cfg(test)]
#[path = "native_application_program_tests.rs"]
mod tests;

#[path = "native_application_program/capture.rs"]
mod capture;
#[path = "native_application_program/changes.rs"]
mod changes;

pub use changes::{UiNativeComponentPresenceChange, UiNativeComponentSemanticTextChange};

#[must_use]
pub struct UiNativeApplicationProgram {
    frames: Box<[UiNativeApplicationFrame]>,
    close_after_program: bool,
}

#[must_use]
pub struct UiNativeApplicationFrame {
    theme_switch: Option<crate::capability::UiThemeDefinitionIdentity>,
    component_presence: Box<[UiNativeComponentPresenceChange]>,
    semantic_text: Box<[UiNativeComponentSemanticTextChange]>,
    start: UiNativeApplicationFrameStart,
    completion: UiNativeApplicationFrameCompletion,
    capture_presented_source_pixels: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeApplicationFrameStart {
    AfterPriorSettlement,
    SupersedingPriorPending,
    AfterHostSurfaceBasisSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeApplicationFrameCompletion {
    Settle,
    CancelAfterExternalSubmission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationProgramDenial {
    ThemeSwitchRequiresPriorSettlement,
    Empty,
    FrameCapacityExceeded,
    ChangeCapacityExceeded,
    InvalidComponentIdentity,
    InvalidSemanticTextSpans,
    SemanticTextUpdateRejected,
    PresentedSourceCaptureCapacityExceeded,
}

impl UiNativeApplicationProgram {
    pub fn new(
        frames: impl IntoIterator<Item = UiNativeApplicationFrame>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let mut frames = frames.into_iter().collect::<Vec<_>>();
        if frames.is_empty() {
            return Err(UiNativeApplicationProgramDenial::Empty);
        }
        if frames.len() > MAXIMUM_FRAMES {
            return Err(UiNativeApplicationProgramDenial::FrameCapacityExceeded);
        }
        if frames.iter().any(|frame| {
            frame.theme_switch.is_some()
                && (frame.starts_by_superseding_pending()
                    || frame.cancels_after_external_submission())
        }) || frames
            .windows(2)
            .any(|pair| pair[0].theme_switch.is_some() && pair[1].starts_by_superseding_pending())
        {
            return Err(UiNativeApplicationProgramDenial::ThemeSwitchRequiresPriorSettlement);
        }
        if frames
            .iter()
            .filter(|frame| frame.capture_presented_source_pixels)
            .count()
            > 1
        {
            return Err(UiNativeApplicationProgramDenial::PresentedSourceCaptureCapacityExceeded);
        }
        let mut revisions = std::collections::HashMap::<Box<str>, u64>::new();
        for frame in &mut frames {
            for change in &mut frame.semantic_text {
                let revision = revisions
                    .entry(change.authored_semantic_identity.clone())
                    .or_default();
                change.expected_revision = *revision;
                *revision = revision
                    .checked_add(1)
                    .ok_or(UiNativeApplicationProgramDenial::ChangeCapacityExceeded)?;
            }
        }
        Ok(Self {
            frames: frames.into_boxed_slice(),
            close_after_program: true,
        })
    }

    pub fn single_frame() -> Self {
        Self {
            frames: Box::new([UiNativeApplicationFrame::present_current()]),
            close_after_program: true,
        }
    }

    pub(crate) fn application_driven() -> Self {
        Self {
            frames: Box::new([]),
            close_after_program: false,
        }
    }

    pub fn remain_open_until_external_close(mut self) -> Self {
        self.close_after_program = false;
        self
    }

    pub(crate) fn frames(&self) -> &[UiNativeApplicationFrame] {
        &self.frames
    }

    pub(crate) const fn closes_after_program(&self) -> bool {
        self.close_after_program
    }
}

impl UiNativeApplicationFrame {
    pub fn present_current() -> Self {
        Self {
            theme_switch: None,
            component_presence: Box::new([]),
            semantic_text: Box::new([]),
            start: UiNativeApplicationFrameStart::AfterPriorSettlement,
            completion: UiNativeApplicationFrameCompletion::Settle,
            capture_presented_source_pixels: false,
        }
    }

    /// Request a theme on this native surface through ordinary observation,
    /// capability admission and rebind settlement when the frame becomes ready.
    pub fn switch_theme(definition: crate::capability::UiThemeDefinitionIdentity) -> Self {
        Self {
            theme_switch: Some(definition),
            ..Self::present_current()
        }
    }

    pub(crate) fn theme_switch(&self) -> Option<&crate::capability::UiThemeDefinitionIdentity> {
        self.theme_switch.as_ref()
    }

    pub fn with_component_presence(
        changes: impl IntoIterator<Item = UiNativeComponentPresenceChange>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let changes = changes.into_iter().collect::<Vec<_>>();
        if changes.len() > MAXIMUM_CHANGES_PER_FRAME {
            return Err(UiNativeApplicationProgramDenial::ChangeCapacityExceeded);
        }
        Ok(Self {
            component_presence: changes.into_boxed_slice(),
            theme_switch: None,
            semantic_text: Box::new([]),
            start: UiNativeApplicationFrameStart::AfterPriorSettlement,
            completion: UiNativeApplicationFrameCompletion::Settle,
            capture_presented_source_pixels: false,
        })
    }

    pub fn with_semantic_text(
        changes: impl IntoIterator<Item = UiNativeComponentSemanticTextChange>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let changes = changes.into_iter().collect::<Vec<_>>();
        if changes.len() > MAXIMUM_CHANGES_PER_FRAME {
            return Err(UiNativeApplicationProgramDenial::ChangeCapacityExceeded);
        }
        Ok(Self {
            component_presence: Box::new([]),
            theme_switch: None,
            semantic_text: changes.into_boxed_slice(),
            start: UiNativeApplicationFrameStart::AfterPriorSettlement,
            completion: UiNativeApplicationFrameCompletion::Settle,
            capture_presented_source_pixels: false,
        })
    }

    pub fn with_component_presence_and_semantic_text(
        presence: impl IntoIterator<Item = UiNativeComponentPresenceChange>,
        semantic_text: impl IntoIterator<Item = UiNativeComponentSemanticTextChange>,
    ) -> Result<Self, UiNativeApplicationProgramDenial> {
        let presence = presence.into_iter().collect::<Vec<_>>();
        let semantic_text = semantic_text.into_iter().collect::<Vec<_>>();
        if presence.len().saturating_add(semantic_text.len()) > MAXIMUM_CHANGES_PER_FRAME {
            return Err(UiNativeApplicationProgramDenial::ChangeCapacityExceeded);
        }
        Ok(Self {
            component_presence: presence.into_boxed_slice(),
            theme_switch: None,
            semantic_text: semantic_text.into_boxed_slice(),
            start: UiNativeApplicationFrameStart::AfterPriorSettlement,
            completion: UiNativeApplicationFrameCompletion::Settle,
            capture_presented_source_pixels: false,
        })
    }

    /// Start this logical successor after its predecessor has entered the
    /// pending presentation phase, without waiting for physical settlement.
    /// The Query presentation owner adjudicates the exact supersession.
    pub fn superseding_pending(mut self) -> Self {
        self.start = UiNativeApplicationFrameStart::SupersedingPriorPending;
        self
    }

    /// Hold this frame until the native host publishes a successor readiness
    /// generation for its surface basis.
    pub fn after_host_surface_basis_successor(mut self) -> Self {
        self.start = UiNativeApplicationFrameStart::AfterHostSurfaceBasisSuccessor;
        self
    }

    /// Request cancellation only after the ordinary native owner has returned
    /// an in-flight presentation capability for a real external submission.
    pub fn cancel_after_external_submission(mut self) -> Self {
        self.completion = UiNativeApplicationFrameCompletion::CancelAfterExternalSubmission;
        self
    }

    pub(crate) fn component_presence(&self) -> &[UiNativeComponentPresenceChange] {
        &self.component_presence
    }

    pub(crate) fn semantic_text(&self) -> &[UiNativeComponentSemanticTextChange] {
        &self.semantic_text
    }

    pub(crate) const fn starts_by_superseding_pending(&self) -> bool {
        matches!(
            self.start,
            UiNativeApplicationFrameStart::SupersedingPriorPending
        )
    }

    pub(crate) const fn awaits_host_surface_basis_successor(&self) -> bool {
        matches!(
            self.start,
            UiNativeApplicationFrameStart::AfterHostSurfaceBasisSuccessor
        )
    }

    pub(crate) const fn cancels_after_external_submission(&self) -> bool {
        matches!(
            self.completion,
            UiNativeApplicationFrameCompletion::CancelAfterExternalSubmission
        )
    }
}
