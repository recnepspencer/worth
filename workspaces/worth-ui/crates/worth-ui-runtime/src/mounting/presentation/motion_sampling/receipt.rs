#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentationReducedMotionPosture {
    NoPreference,
    Reduce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentationMotionSamplePosture {
    Delayed,
    Active,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPresentationMotionTerminalRequest {
    track: crate::runtime::motion::UiMotionTrackIdentity,
    cause: crate::runtime::motion::UiMotionTerminalCause,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPresentationMotionSamplingCost {
    tracks_considered: u64,
    hit_index_work: crate::mounting::hit_test_work::UiHitTestSpatialWork,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentationMotionSampleReceipt {
    track: crate::runtime::motion::UiMotionTrackIdentity,
    target: crate::runtime::motion::UiMotionTargetIdentity,
    tick: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    base_geometry: Option<crate::runtime::motion::UiMotionSemanticGeometry>,
    geometry: Option<super::UiPresentationSampledGeometry>,
    opacity_units: u16,
    hit_test_visible: bool,
    damage: super::UiPresentationMotionDamage,
    posture: UiPresentationMotionSamplePosture,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiPresentationMotionSamplingReceipt {
    pub(super) samples: Box<[UiPresentationMotionSampleReceipt]>,
    terminals: Box<[UiPresentationMotionTerminalRequest]>,
    cost: UiPresentationMotionSamplingCost,
    hit_transition: Option<crate::mounting::UiCommittedPresentedHitTransition>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentationMotionInstallationReceipt {
    sample: Option<UiPresentationMotionSampleReceipt>,
    terminal: Option<UiPresentationMotionTerminalRequest>,
}

impl UiPresentationMotionTerminalRequest {
    pub(super) const fn new(
        track: crate::runtime::motion::UiMotionTrackIdentity,
        cause: crate::runtime::motion::UiMotionTerminalCause,
    ) -> Self {
        Self { track, cause }
    }

    pub(crate) const fn track(self) -> crate::runtime::motion::UiMotionTrackIdentity {
        self.track
    }

    pub(crate) const fn cause(self) -> crate::runtime::motion::UiMotionTerminalCause {
        self.cause
    }
}

impl UiPresentationMotionSamplingCost {
    pub(super) fn new(tracks: usize) -> Self {
        Self {
            tracks_considered: tracks as u64,
            hit_index_work: Default::default(),
        }
    }

    pub(in crate::mounting) const fn hit_index_work(
        self,
    ) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        self.hit_index_work
    }

    pub(crate) const fn tracks_considered(self) -> u64 {
        self.tracks_considered
    }
}

impl UiPresentationMotionSampleReceipt {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        track: crate::runtime::motion::UiMotionTrackIdentity,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        base_geometry: Option<crate::runtime::motion::UiMotionSemanticGeometry>,
        geometry: Option<super::UiPresentationSampledGeometry>,
        opacity_units: u16,
        hit_test_visible: bool,
        damage: super::UiPresentationMotionDamage,
        posture: UiPresentationMotionSamplePosture,
    ) -> Self {
        Self {
            track,
            target,
            tick,
            presentation,
            base_geometry,
            geometry,
            opacity_units,
            hit_test_visible,
            damage,
            posture,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_track_sample(
        track: crate::runtime::motion::UiCommittedMotionTrack,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        components: Option<[f32; 4]>,
        opacity_units: u16,
        posture: UiPresentationMotionSamplePosture,
        damage: super::UiPresentationMotionDamage,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        if presentation.host_surface() != track.successor_presentation().host_surface() {
            return Err(super::UiPresentationGeometrySamplingDenial::PresentationSurfaceChanged);
        }
        if presentation.binding() != track.successor_presentation().binding() {
            return Err(super::UiPresentationGeometrySamplingDenial::PresentationBindingChanged);
        }
        let semantic_basis = track.successor_geometry();
        let geometry = components
            .zip(semantic_basis)
            .map(|(components, semantic_basis)| {
                super::UiPresentationSampledGeometry::from_motion_sample(
                    track.target(),
                    track.successor_revision(),
                    track.successor_presentation(),
                    semantic_basis,
                    components,
                    presentation,
                )
            })
            .transpose()?;
        if components.is_some() != semantic_basis.is_some() {
            return Err(super::UiPresentationGeometrySamplingDenial::MissingSemanticBasis);
        }
        let hit_test_visible = track.successor_visible();
        Ok(Self::new(
            track.identity(),
            track.target(),
            tick,
            presentation,
            semantic_basis,
            geometry,
            opacity_units,
            hit_test_visible,
            damage,
            posture,
        ))
    }

    pub(crate) const fn track(self) -> crate::runtime::motion::UiMotionTrackIdentity {
        self.track
    }
    pub(crate) const fn target(self) -> crate::runtime::motion::UiMotionTargetIdentity {
        self.target
    }
    pub(crate) const fn tick(self) -> u64 {
        self.tick
    }
    pub(crate) const fn presentation_basis(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
    pub(crate) const fn base_geometry(
        self,
    ) -> Option<crate::runtime::motion::UiMotionSemanticGeometry> {
        self.base_geometry
    }
    pub(crate) const fn geometry(self) -> Option<super::UiPresentationSampledGeometry> {
        self.geometry
    }
    pub(crate) const fn opacity_units(self) -> u16 {
        self.opacity_units
    }
    pub(crate) const fn hit_test_visible(self) -> bool {
        self.hit_test_visible
    }
    pub(crate) const fn damage(self) -> super::UiPresentationMotionDamage {
        self.damage
    }
    pub(crate) const fn posture(self) -> UiPresentationMotionSamplePosture {
        self.posture
    }

    pub(in crate::mounting::presentation) fn with_presentation_basis(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<Self, super::UiPresentationGeometrySamplingDenial> {
        if presentation.host_surface() != self.presentation.host_surface() {
            return Err(super::UiPresentationGeometrySamplingDenial::PresentationSurfaceChanged);
        }
        if presentation.binding() != self.presentation.binding() {
            return Err(super::UiPresentationGeometrySamplingDenial::PresentationBindingChanged);
        }
        self.geometry = self
            .geometry
            .map(|geometry| geometry.with_presentation_basis(presentation))
            .transpose()?;
        self.presentation = presentation;
        Ok(self)
    }

    pub(super) fn rebind_presentation_basis(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        self.geometry = self
            .geometry
            .map(|geometry| geometry.rebind_presentation_basis(presentation));
        self.presentation = presentation;
        self
    }
}

impl UiPresentationMotionSamplingReceipt {
    pub(crate) fn take_hit_transition(
        &mut self,
    ) -> Option<crate::mounting::UiCommittedPresentedHitTransition> {
        self.hit_transition.take()
    }
    pub(in crate::mounting) fn record_hit_transition(
        &mut self,
        transition: Option<crate::mounting::UiCommittedPresentedHitTransition>,
    ) {
        self.hit_transition = transition;
    }

    pub(in crate::mounting) fn record_hit_index_work(
        &mut self,
        work: crate::mounting::hit_test_work::UiHitTestSpatialWork,
    ) {
        self.cost.hit_index_work.merge(work);
    }

    pub(super) fn new(
        samples: Vec<UiPresentationMotionSampleReceipt>,
        terminals: Vec<UiPresentationMotionTerminalRequest>,
        tracks_considered: usize,
    ) -> Self {
        let cost = UiPresentationMotionSamplingCost::new(tracks_considered);
        Self {
            samples: samples.into_boxed_slice(),
            terminals: terminals.into_boxed_slice(),
            cost,
            hit_transition: None,
        }
    }

    pub(crate) fn samples(&self) -> &[UiPresentationMotionSampleReceipt] {
        &self.samples
    }
    pub(crate) fn terminals(&self) -> &[UiPresentationMotionTerminalRequest] {
        &self.terminals
    }
    pub(crate) const fn cost(&self) -> UiPresentationMotionSamplingCost {
        self.cost
    }
}

impl UiPresentationMotionInstallationReceipt {
    pub(super) const fn new(
        sample: Option<UiPresentationMotionSampleReceipt>,
        terminal: Option<UiPresentationMotionTerminalRequest>,
    ) -> Self {
        Self { sample, terminal }
    }

    #[cfg(test)]
    pub(crate) const fn sample(self) -> Option<UiPresentationMotionSampleReceipt> {
        self.sample
    }
    pub(crate) const fn terminal(self) -> Option<UiPresentationMotionTerminalRequest> {
        self.terminal
    }
}
