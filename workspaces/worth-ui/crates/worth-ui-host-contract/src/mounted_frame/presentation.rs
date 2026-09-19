#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiMountedEffectFamily {
    RecordedProjection,
    NativePaint,
    Accessibility,
    Diagnostic,
    IdentityOverlay,
    CanvasSpatial,
    Realtime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedCompletedEffects {
    families: Box<[UiMountedEffectFamily]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedSurfacePresentationCompletion {
    mode: crate::UiHostSurfacePresentationMode,
    epoch: crate::UiHostPresentationEpoch,
    effects: UiMountedCompletedEffects,
    cost: super::presentation_cost::UiHostPresentationCostReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostSurfacePresentationDenial {
    AdapterDeclined,
    ExternalTimeout,
    SurfaceOccluded,
    ExternalValidationFailed,
    TextAtlasPresentationDeferred,
    CancelledBeforeEffects,
    UnsupportedPresentationMode(crate::UiHostSurfacePresentationMode),
    UnsupportedEffect(UiMountedEffectFamily),
    Protocol(crate::UiHostProtocolDenial),
    ProtocolChanged,
    CapabilityGenerationChanged,
    CapabilityProfileChanged,
    SurfaceBindingChanged,
    ReconstructionRequired,
    StalePredecessor,
    MalformedProjection,
    DeadlineExpired,
    CapacityExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPresentationDeadline {
    tick: u64,
}

pub struct UiMountedFrameConsumptionView<'frame> {
    authority: Rc<()>,
    host_session_identity: u64,
    protocol: crate::UiHostProtocolAgreement,
    capability_generation: crate::WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    attempt: crate::UiMountedPresentationAttemptIdentity,
    deadline: UiPresentationDeadline,
    requirement: crate::UiMountedSurfaceBindingRequirement,
    presentation_work: super::presentation_work::UiMountedPresentationWorkView<'frame>,
    appearance_work: Option<&'frame crate::UiMountedAppearancePresentationWork>,
    qualified_text: &'frame dyn crate::UiMountedQualifiedTextResolver,
    text_raster_work: Option<&'frame UiMountedTextRasterWork<'frame>>,
}

#[doc(hidden)]
pub struct UiMountedFrameConsumptionInput<'frame> {
    pub authority: Rc<()>,
    pub host_session_identity: u64,
    pub protocol: crate::UiHostProtocolAgreement,
    pub capability_generation: crate::WorthUiHostCapabilityObservationGeneration,
    pub capability_profile_digest: u64,
    pub attempt: crate::UiMountedPresentationAttemptIdentity,
    pub deadline: UiPresentationDeadline,
    pub requirement: crate::UiMountedSurfaceBindingRequirement,
    pub presentation_work: super::presentation_work::UiMountedPresentationWorkView<'frame>,
    pub appearance_work: Option<&'frame crate::UiMountedAppearancePresentationWork>,
    pub qualified_text: &'frame dyn crate::UiMountedQualifiedTextResolver,
    pub text_raster_work: Option<&'frame UiMountedTextRasterWork<'frame>>,
}

pub struct UiHostPresentationCompletionToken {
    identity: u64,
    authority: Rc<()>,
    progress_class: UiHostPresentationProgressClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedSurfacePresentationSupersession {
    cost: super::presentation_cost::UiHostPresentationCostReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostPresentationProgressClass {
    PhysicalSurface,
    TextAtlas,
}

impl<'frame> UiMountedFrameConsumptionView<'frame> {
    #[doc(hidden)]
    pub fn from_inert_mechanics(input: UiMountedFrameConsumptionInput<'frame>) -> Self {
        let work_affinity = input.presentation_work.affinity();
        assert_eq!(
            (
                work_affinity.surface(),
                work_affinity.binding(),
                work_affinity.baseline(),
            ),
            (
                input.requirement.semantic_surface(),
                input.requirement.binding(),
                input.requirement.baseline(),
            ),
            "presentation work must carry the exact consumed surface binding and baseline"
        );
        assert_eq!(
            (input.capability_generation, input.capability_profile_digest,),
            (
                input.requirement.capability_generation(),
                input.requirement.capability_profile_digest(),
            ),
            "presentation work consumption must use the registered profile generation"
        );
        UiMountedFrameConsumptionView {
            authority: input.authority,
            host_session_identity: input.host_session_identity,
            protocol: input.protocol,
            capability_generation: input.capability_generation,
            capability_profile_digest: input.capability_profile_digest,
            attempt: input.attempt,
            deadline: input.deadline,
            requirement: input.requirement,
            presentation_work: input.presentation_work,
            appearance_work: input.appearance_work,
            qualified_text: input.qualified_text,
            text_raster_work: input.text_raster_work,
        }
    }

    #[doc(hidden)]
    pub fn issued_by_runtime(&self, seal: &Rc<()>) -> bool {
        Rc::ptr_eq(&self.authority, seal)
    }
}

impl<'frame> UiMountedFrameConsumptionView<'frame> {
    pub fn host_session_identity(&self) -> u64 {
        self.host_session_identity
    }

    pub fn host_presentation_lineage(&self) -> Option<crate::UiHostPresentationLineageIdentity> {
        crate::UiHostPresentationLineageIdentity::from_host_session(self.host_session_identity)
    }

    pub fn protocol(&self) -> crate::UiHostProtocolAgreement {
        self.protocol
    }

    pub fn capability_generation(&self) -> crate::WorthUiHostCapabilityObservationGeneration {
        self.capability_generation
    }

    pub fn capability_profile_digest(&self) -> u64 {
        self.capability_profile_digest
    }

    pub fn attempt(&self) -> crate::UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub fn deadline(&self) -> UiPresentationDeadline {
        self.deadline
    }

    pub fn requirement(&self) -> crate::UiMountedSurfaceBindingRequirement {
        self.requirement
    }

    pub fn presentation_work(
        &self,
    ) -> super::presentation_work::UiMountedPresentationWorkView<'frame> {
        self.presentation_work
    }

    pub fn appearance_work(&self) -> Option<&crate::UiMountedAppearancePresentationWork> {
        self.appearance_work
    }

    pub fn qualified_text_layout(
        &self,
        mechanic: &crate::UiMountedSemanticTextMechanic,
    ) -> Option<crate::UiQualifiedTextLayoutView<'frame>> {
        let view = self
            .qualified_text
            .resolve(mechanic.qualified_layout_identity())?;
        (view.identity() == mechanic.qualified_layout_identity()
            && view.request_identity() == mechanic.qualified_layout_request()
            && view.profile_generation() == mechanic.qualified_layout_profile()
            && view.font_collection_generation() == mechanic.qualified_layout_fonts()
            && view.text_scale_generation() == mechanic.qualified_layout_scale())
        .then_some(view)
    }

    #[doc(hidden)]
    pub fn text_raster_work(&self) -> Option<&UiMountedTextRasterWork<'frame>> {
        self.text_raster_work
    }

    pub fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.presentation_work.affinity().successor()
    }

    pub fn surface(&self) -> crate::UiSemanticSurfaceIdentity {
        self.presentation_work.affinity().surface()
    }

    pub fn binding(&self) -> crate::UiSurfaceBindingGeneration {
        self.presentation_work.affinity().binding()
    }

    pub fn content_generation(&self) -> crate::UiMountedContentGeneration {
        self.presentation_work.affinity().content()
    }

    pub fn issue_completion_token(&self) -> UiHostPresentationCompletionToken {
        self.issue_completion_token_for(UiHostPresentationProgressClass::PhysicalSurface)
    }

    pub fn issue_text_atlas_completion_token(&self) -> UiHostPresentationCompletionToken {
        self.issue_completion_token_for(UiHostPresentationProgressClass::TextAtlas)
    }

    fn issue_completion_token_for(
        &self,
        progress_class: UiHostPresentationProgressClass,
    ) -> UiHostPresentationCompletionToken {
        static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);
        let identity = NEXT_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("presentation completion token identity exhausted");
        UiHostPresentationCompletionToken {
            identity,
            authority: Rc::clone(&self.authority),
            progress_class,
        }
    }
}

impl UiHostPresentationCompletionToken {
    #[doc(hidden)]
    pub fn issued_by_runtime(&self, seal: &Rc<()>) -> bool {
        Rc::ptr_eq(&self.authority, seal)
    }

    pub fn diagnostic_value(&self) -> u64 {
        self.identity
    }

    pub const fn progress_class(&self) -> UiHostPresentationProgressClass {
        self.progress_class
    }
}

impl std::fmt::Debug for UiHostPresentationCompletionToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiHostPresentationCompletionToken")
            .field("identity", &self.identity)
            .field("progress_class", &self.progress_class)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum UiHostSurfacePresentationOutcome {
    RejectedBeforeEffects(UiHostSurfacePresentationDenial),
    Presented(UiMountedSurfacePresentationCompletion),
    InFlight(crate::UiHostPresentationCompletionToken),
    PresentationIndeterminate,
}

#[derive(Debug)]
pub enum UiHostSurfaceInFlightCompletion {
    Pending(UiHostPresentationCompletionToken),
    RejectedBeforeEffects(UiHostSurfacePresentationDenial),
    Presented(UiMountedSurfacePresentationCompletion),
    Superseded(UiMountedSurfacePresentationSupersession),
    PresentationIndeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostSurfaceCancellationOutcome {
    CancelledBeforeEffects,
    EffectsMayHaveBegun,
}

impl UiMountedCompletedEffects {
    pub fn new(mut families: Vec<UiMountedEffectFamily>) -> Self {
        families.sort();
        families.dedup();
        Self {
            families: families.into_boxed_slice(),
        }
    }

    pub fn families(&self) -> &[UiMountedEffectFamily] {
        &self.families
    }
}

impl UiMountedSurfacePresentationCompletion {
    pub fn new(
        mode: crate::UiHostSurfacePresentationMode,
        epoch: crate::UiHostPresentationEpoch,
        effects: UiMountedCompletedEffects,
        cost: super::presentation_cost::UiHostPresentationCostReport,
    ) -> Self {
        Self {
            mode,
            epoch,
            effects,
            cost,
        }
    }

    pub fn mode(&self) -> crate::UiHostSurfacePresentationMode {
        self.mode
    }

    pub fn epoch(&self) -> crate::UiHostPresentationEpoch {
        self.epoch
    }

    pub fn effects(&self) -> &UiMountedCompletedEffects {
        &self.effects
    }

    pub fn cost(&self) -> super::presentation_cost::UiHostPresentationCostReport {
        self.cost
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::UiHostPresentationEpoch,
        UiMountedCompletedEffects,
        super::presentation_cost::UiHostPresentationCostReport,
    ) {
        (self.epoch, self.effects, self.cost)
    }
}

impl UiMountedSurfacePresentationSupersession {
    pub fn observed(cost: super::presentation_cost::UiHostPresentationCostReport) -> Self {
        Self { cost }
    }

    pub fn cost(self) -> super::presentation_cost::UiHostPresentationCostReport {
        self.cost
    }
}

impl UiPresentationDeadline {
    pub const fn at_tick(tick: u64) -> Self {
        Self { tick }
    }

    pub const fn tick(self) -> u64 {
        self.tick
    }

    pub const fn expired_at(self, now: u64) -> bool {
        now >= self.tick
    }
}
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
#[path = "presentation/text_raster_work.rs"]
mod text_raster_work;

pub use text_raster_work::{
    UiMountedTextDemandValidationCost, UiMountedTextDemandValidationDenial,
    UiMountedTextRasterCallback, UiMountedTextRasterWork,
};
