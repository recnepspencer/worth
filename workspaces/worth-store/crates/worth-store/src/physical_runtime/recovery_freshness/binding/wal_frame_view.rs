//! Borrowed C9 frames with the existing retained-tail and covered-prefix selection.

use worth_store_recovery_physics::SelectedPhysicalWalTail;

use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, IntegrityAdmittedRecoveryWalSegment,
};

/// A concrete read-only view of already Integrity-admitted WAL storage.
/// Cloning copies references only; no frame or payload backing is constructed.
#[derive(Clone, Copy)]
pub struct IntegrityAdmittedRecoveryWalFrameView<'a> {
    source: FrameSource<'a>,
}

#[derive(Clone, Copy)]
enum FrameSource<'a> {
    Frames(&'a [IntegrityAdmittedRecoveryWalFrame]),
    Selected {
        segments: &'a [IntegrityAdmittedRecoveryWalSegment],
        tail: &'a SelectedPhysicalWalTail,
    },
}

impl<'a> IntegrityAdmittedRecoveryWalFrameView<'a> {
    pub const fn from_frames(frames: &'a [IntegrityAdmittedRecoveryWalFrame]) -> Self {
        Self {
            source: FrameSource::Frames(frames),
        }
    }

    /// Includes every admitted frame from selected retained-tail segments and
    /// checkpoint-covered segments, including covered retirement obligations.
    pub const fn from_selected_segments(
        segments: &'a [IntegrityAdmittedRecoveryWalSegment],
        tail: &'a SelectedPhysicalWalTail,
    ) -> Self {
        Self {
            source: FrameSource::Selected { segments, tail },
        }
    }

    pub fn iter(self) -> impl Iterator<Item = &'a IntegrityAdmittedRecoveryWalFrame> + Clone + 'a {
        match self.source {
            FrameSource::Frames(frames) => FrameIterator {
                frames: frames.iter(),
                segments: [].iter(),
                selected: None,
            },
            FrameSource::Selected { segments, tail } => FrameIterator {
                frames: [].iter(),
                segments: segments.iter(),
                selected: Some(tail),
            },
        }
    }
}

#[derive(Clone)]
struct FrameIterator<'a> {
    frames: std::slice::Iter<'a, IntegrityAdmittedRecoveryWalFrame>,
    segments: std::slice::Iter<'a, IntegrityAdmittedRecoveryWalSegment>,
    selected: Option<&'a SelectedPhysicalWalTail>,
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = &'a IntegrityAdmittedRecoveryWalFrame;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(frame) = self.frames.next() {
                return Some(frame);
            }
            let segment = self.segments.next()?;
            let identity = segment.inspection().identity();
            let selected = self.selected?;
            if selected
                .segments()
                .iter()
                .any(|candidate| candidate.identity() == identity)
                || selected
                    .checkpoint_covered()
                    .iter()
                    .any(|covered| covered.identity() == identity)
            {
                self.frames = segment.frames().iter();
            }
        }
    }
}
