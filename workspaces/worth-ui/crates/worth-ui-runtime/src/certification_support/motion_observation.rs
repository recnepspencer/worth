use crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingDenial;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMotionPresentationCertificationSnapshot {
    active_tracks: usize,
    retained_samples: usize,
    last_tick: Option<u64>,
    semantic_publications: u64,
    sample: Option<UiMotionSampleCertification>,
    sampling_ready: bool,
    hit_test_truth_available: bool,
    sampling_denials: u64,
    last_denial: Option<UiPresentationMotionSamplingDenial>,
}

/// The retained Motion sample; its geometry, when resolved, carries the
/// presentation basis it was placed under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMotionSampleCertification {
    pub(crate) placement: Option<(
        [f32; 4],
        worth_ui_host_contract::UiHostObservationPresentationBasis,
    )>,
    pub(crate) opacity_units: u16,
    pub(crate) hit_test_visible: bool,
}

pub trait WorthUiMotionPresentationCertificationExt {
    fn inspect_motion_presentation_for_certification(
        &self,
    ) -> UiMotionPresentationCertificationSnapshot;

    fn complete_motion_sample_for_certification(&mut self);
}

impl WorthUiMotionPresentationCertificationExt for crate::facade::WorthUiActiveApplicationSession {
    fn inspect_motion_presentation_for_certification(
        &self,
    ) -> UiMotionPresentationCertificationSnapshot {
        crate::facade::WorthUiActiveApplicationSession::inspect_motion_presentation_for_certification(
            self,
        )
    }

    fn complete_motion_sample_for_certification(&mut self) {
        crate::facade::WorthUiActiveApplicationSession::complete_motion_sample_presentation(self);
    }
}

impl UiMotionPresentationCertificationSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        active_tracks: usize,
        retained_samples: usize,
        last_tick: Option<u64>,
        semantic_publications: u64,
        sample: Option<UiMotionSampleCertification>,
        sampling_ready: bool,
        hit_test_truth_available: bool,
        sampling_denials: u64,
        last_denial: Option<UiPresentationMotionSamplingDenial>,
    ) -> Self {
        Self {
            active_tracks,
            retained_samples,
            last_tick,
            semantic_publications,
            sample,
            sampling_ready,
            hit_test_truth_available,
            sampling_denials,
            last_denial,
        }
    }

    pub const fn active_tracks(self) -> usize {
        self.active_tracks
    }
    pub const fn retained_samples(self) -> usize {
        self.retained_samples
    }
    pub const fn last_tick(self) -> Option<u64> {
        self.last_tick
    }
    pub const fn semantic_publications(self) -> u64 {
        self.semantic_publications
    }
    pub fn geometry(self) -> Option<[f32; 4]> {
        self.sample
            .and_then(|sample| sample.placement)
            .map(|(geometry, _)| geometry)
    }
    pub fn opacity_units(self) -> Option<u16> {
        self.sample.map(|sample| sample.opacity_units)
    }
    pub fn hit_test_visible(self) -> Option<bool> {
        self.sample.map(|sample| sample.hit_test_visible)
    }
    pub fn presentation(
        self,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.sample
            .and_then(|sample| sample.placement)
            .map(|(_, presentation)| presentation)
    }
    pub const fn sampling_denials(self) -> u64 {
        self.sampling_denials
    }
    pub const fn sampling_ready(self) -> bool {
        self.sampling_ready
    }
    pub const fn hit_test_truth_available(self) -> bool {
        self.hit_test_truth_available
    }
    pub const fn last_denial_was_non_monotonic(self) -> bool {
        matches!(
            self.last_denial,
            Some(UiPresentationMotionSamplingDenial::NonMonotonicTick)
        )
    }
    pub const fn last_denial_was_presentation_truth_unavailable(self) -> bool {
        matches!(
            self.last_denial,
            Some(UiPresentationMotionSamplingDenial::PresentationTruthUnavailable)
        )
    }
}
