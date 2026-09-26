use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use worth_ui_host_contract::{
    UiHostSessionReleaseReceipt, UiMeasurementEvidenceFamily, UiMeasurementRequestIdentity,
    WorthUiHostCapabilityObservationGeneration, WorthUiHostCapabilityReport,
};
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

use crate::host::adapter::{
    UiHostAdapterSessionAuthority, UiHostSessionReleaseIndeterminate, UiHostSessionReleaseOutcome,
    WorthUiOperationalHostAdapter,
};
use crate::host::{
    UiHostMeasurementCollectionInput, UiHostMeasurementNeed, UiHostMeasurementNormalizationContext,
};

static NEXT_HOST_SESSION_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiHostSessionIdentity {
    value: u64,
}

#[derive(Clone)]
pub struct WorthUiHostMeasurementCapability {
    session_identity: WorthUiHostSessionIdentity,
    capability_report: WorthUiHostCapabilityReport,
    adapter: Rc<dyn WorthUiOperationalHostAdapter>,
}

pub struct WorthUiHostMeasurementSessionInput {
    identity: UiMeasurementRequestIdentity,
    evidence_family: UiMeasurementEvidenceFamily,
    need: UiHostMeasurementNeed,
    evidence_generation: UiEvidenceAuthorityGeneration,
    normalization_context: UiHostMeasurementNormalizationContext,
}

pub(crate) struct WorthUiHostSessionAuthority {
    identity: WorthUiHostSessionIdentity,
    protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    measurement_capability: WorthUiHostMeasurementCapability,
    mounted_presentation_lease: crate::mounting::presentation::UiMountedPresentationLease,
    adapter_authority: UiHostAdapterSessionAuthority,
    adapter_session_released: bool,
}

/// Move-only authority retained when terminal host release cannot yet be proved.
pub struct WorthUiHostSessionReleaseRecovery {
    authority: Box<WorthUiHostSessionAuthority>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiHostEffectPort<'session> {
    adapter: &'session dyn WorthUiOperationalHostAdapter,
    authority: &'session UiHostAdapterSessionAuthority,
}

/// Narrow immutable host authority carried into plan construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiHostPlanBinding {
    session_identity: WorthUiHostSessionIdentity,
    observation_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    canvas_spatial_execution_supported: bool,
    realtime_overlay_execution_supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiHostSessionActivationDenial {
    IdentityExhausted,
    Protocol(worth_ui_host_contract::UiHostProtocolDenial),
    MountedPresentationLease(crate::mounting::presentation::UiMountedPresentationLeaseDenial),
    ObservationSession(worth_ui_host_contract::UiHostObservationSessionRegistrationDenial),
}

impl WorthUiHostSessionAuthority {
    pub(crate) fn activate(
        plan: &crate::facade::prepared_application_authority::WorthUiHostSessionPlan,
    ) -> Result<Self, WorthUiHostSessionActivationDenial> {
        let protocol = match plan.protocol_contract().negotiate() {
            worth_ui_host_contract::UiHostProtocolNegotiation::Compatible(agreement) => agreement,
            worth_ui_host_contract::UiHostProtocolNegotiation::Incompatible(denial) => {
                return Err(WorthUiHostSessionActivationDenial::Protocol(denial));
            }
        };
        let value = NEXT_HOST_SESSION_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                next_host_session_identity_value(current)
            })
            .map_err(|_| WorthUiHostSessionActivationDenial::IdentityExhausted)?;
        let identity = WorthUiHostSessionIdentity { value };
        let adapter_authority = UiHostAdapterSessionAuthority::activate(value);
        let mounted_presentation_lease = adapter_authority
            .claim_mounted_presentation_lease()
            .map_err(WorthUiHostSessionActivationDenial::MountedPresentationLease)?;
        let capability_report = plan
            .capability_report()
            .clone()
            .with_observation_generation(WorthUiHostCapabilityObservationGeneration::new(value));
        let adapter = plan.adapter();
        adapter
            .open_host_session(&adapter_authority)
            .map_err(WorthUiHostSessionActivationDenial::ObservationSession)?;
        Ok(Self {
            identity,
            protocol,
            measurement_capability: WorthUiHostMeasurementCapability {
                session_identity: identity,
                capability_report,
                adapter,
            },
            mounted_presentation_lease,
            adapter_authority,
            adapter_session_released: false,
        })
    }

    pub(crate) fn identity(&self) -> WorthUiHostSessionIdentity {
        self.identity
    }

    pub(crate) fn measurement_capability(&self) -> WorthUiHostMeasurementCapability {
        self.measurement_capability.clone()
    }

    pub(crate) fn output_adapter(&self) -> &dyn WorthUiOperationalHostAdapter {
        self.measurement_capability.adapter()
    }

    pub(crate) fn effect_port(&self) -> UiHostEffectPort<'_> {
        UiHostEffectPort {
            adapter: self.output_adapter(),
            authority: &self.adapter_authority,
        }
    }

    pub(crate) fn drain_observations(
        &self,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationDrain,
        worth_ui_host_contract::UiHostObservationDrainDenial,
    > {
        self.output_adapter()
            .drain_host_observations(&self.adapter_authority)
    }

    pub(crate) fn install_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.output_adapter()
            .install_input_recipient(&self.adapter_authority, binding)
    }

    pub(crate) fn clear_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.output_adapter()
            .clear_input_recipient(&self.adapter_authority, binding)
    }

    pub(crate) fn release_adapter_session(&mut self) -> UiHostSessionReleaseOutcome {
        if self.adapter_session_released {
            return crate::host::adapter::UiHostSessionReleaseOutcome::Released(
                crate::host::adapter::UiHostSessionReleaseReceipt::released(self.identity.value, 0),
            );
        }
        let outcome = self
            .output_adapter()
            .release_host_session(&self.adapter_authority);
        let validated = match outcome {
            UiHostSessionReleaseOutcome::Released(receipt)
                if receipt.host_session_identity() == self.identity.value =>
            {
                outcome
            }
            UiHostSessionReleaseOutcome::ReleaseIndeterminate(indeterminate)
                if indeterminate.host_session_identity() == self.identity.value =>
            {
                outcome
            }
            UiHostSessionReleaseOutcome::Released(_)
            | UiHostSessionReleaseOutcome::ReleaseIndeterminate(_) => {
                UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                    UiHostSessionReleaseIndeterminate::after_effects_may_have_begun(
                        self.identity.value,
                    ),
                )
            }
        };
        self.adapter_session_released =
            matches!(validated, UiHostSessionReleaseOutcome::Released(_));
        validated
    }

    pub(crate) fn capability_report(&self) -> &WorthUiHostCapabilityReport {
        self.measurement_capability.capability_report()
    }

    pub(crate) fn protocol(&self) -> worth_ui_host_contract::UiHostProtocolAgreement {
        self.protocol
    }

    pub(crate) fn mounted_presentation_lease(
        &self,
    ) -> &crate::mounting::presentation::UiMountedPresentationLease {
        &self.mounted_presentation_lease
    }

    pub(crate) fn plan_binding(&self) -> WorthUiHostPlanBinding {
        WorthUiHostPlanBinding::from_session(self)
    }
}

impl WorthUiHostSessionReleaseRecovery {
    pub(crate) fn retain(authority: WorthUiHostSessionAuthority) -> Self {
        Self {
            authority: Box::new(authority),
        }
    }

    pub fn retry(mut self) -> Result<UiHostSessionReleaseReceipt, Self> {
        match self.authority.release_adapter_session() {
            UiHostSessionReleaseOutcome::Released(receipt) => Ok(receipt),
            UiHostSessionReleaseOutcome::ReleaseIndeterminate(_) => Err(self),
        }
    }

    pub fn host_session_identity(&self) -> WorthUiHostSessionIdentity {
        self.authority.identity()
    }
}

impl std::fmt::Debug for WorthUiHostSessionReleaseRecovery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthUiHostSessionReleaseRecovery")
            .field("host_session_identity", &self.host_session_identity())
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthUiHostSessionReleaseRecovery {
    fn eq(&self, other: &Self) -> bool {
        self.host_session_identity() == other.host_session_identity()
    }
}

impl Eq for WorthUiHostSessionReleaseRecovery {}

impl<'session> UiHostEffectPort<'session> {
    pub(crate) fn adapter(self) -> &'session dyn WorthUiOperationalHostAdapter {
        self.adapter
    }

    pub(crate) fn authority(self) -> &'session UiHostAdapterSessionAuthority {
        self.authority
    }
}

impl Drop for WorthUiHostSessionAuthority {
    fn drop(&mut self) {
        if !self.adapter_session_released {
            let _ = self.release_adapter_session();
        }
    }
}

impl WorthUiHostPlanBinding {
    fn from_session(session: &WorthUiHostSessionAuthority) -> Self {
        let report = session.measurement_capability.capability_report();
        let required = [
            worth_ui_host_contract::WorthUiHostCapability::CanvasSpatialDraw,
            worth_ui_host_contract::WorthUiHostCapability::CanvasSpatialHitTest,
            worth_ui_host_contract::WorthUiHostCapability::CanvasSpatialOverlay,
            worth_ui_host_contract::WorthUiHostCapability::CanvasSpatialToolState,
            worth_ui_host_contract::WorthUiHostCapability::CanvasSpatialRenderResource,
        ];
        let realtime_required = [
            worth_ui_host_contract::WorthUiHostCapability::RealtimeOverlayDraw,
            worth_ui_host_contract::WorthUiHostCapability::RealtimeOverlaySurface,
            worth_ui_host_contract::WorthUiHostCapability::RealtimeOverlayHook,
        ];
        let mounted_recording =
            report.supports(worth_ui_host_contract::WorthUiHostCapability::MountedFrameRecording);
        Self {
            session_identity: session.identity,
            observation_generation: report.observation_generation(),
            capability_profile_digest: report.profile_identity_digest(),
            canvas_spatial_execution_supported: mounted_recording
                || required
                    .into_iter()
                    .all(|capability| report.supports(capability)),
            realtime_overlay_execution_supported: mounted_recording
                || realtime_required
                    .into_iter()
                    .all(|capability| report.supports(capability)),
        }
    }

    pub(crate) fn session_identity(self) -> WorthUiHostSessionIdentity {
        self.session_identity
    }

    pub(crate) fn observation_generation(self) -> WorthUiHostCapabilityObservationGeneration {
        self.observation_generation
    }

    pub(crate) fn capability_profile_digest(self) -> u64 {
        self.capability_profile_digest
    }

    pub(crate) fn canvas_spatial_execution_supported(self) -> bool {
        self.canvas_spatial_execution_supported
    }

    pub(crate) fn realtime_overlay_execution_supported(self) -> bool {
        self.realtime_overlay_execution_supported
    }

    pub(crate) fn shares_session_with(self, other: Self) -> bool {
        self.session_identity == other.session_identity
    }

    pub(crate) fn executable_contract_matches(self, other: Self) -> bool {
        self.canvas_spatial_execution_supported == other.canvas_spatial_execution_supported
            && self.realtime_overlay_execution_supported
                == other.realtime_overlay_execution_supported
    }
}

fn next_host_session_identity_value(current: u64) -> Option<u64> {
    current.checked_add(1)
}

impl WorthUiHostSessionIdentity {
    pub fn as_u64(self) -> u64 {
        self.value
    }
}

impl WorthUiHostMeasurementCapability {
    pub fn session_identity(&self) -> WorthUiHostSessionIdentity {
        self.session_identity
    }

    pub fn observation_generation(&self) -> WorthUiHostCapabilityObservationGeneration {
        self.capability_report.observation_generation()
    }

    pub(crate) fn adapter(&self) -> &dyn WorthUiOperationalHostAdapter {
        self.adapter.as_ref()
    }

    pub fn capability_report(&self) -> &WorthUiHostCapabilityReport {
        &self.capability_report
    }
}

impl WorthUiHostMeasurementSessionInput {
    pub fn new(
        identity: UiMeasurementRequestIdentity,
        evidence_family: UiMeasurementEvidenceFamily,
        need: UiHostMeasurementNeed,
        evidence_generation: UiEvidenceAuthorityGeneration,
        normalization_context: UiHostMeasurementNormalizationContext,
    ) -> Self {
        Self {
            identity,
            evidence_family,
            need,
            evidence_generation,
            normalization_context,
        }
    }

    pub(crate) fn bind_report(
        self,
        capability_report: &WorthUiHostCapabilityReport,
    ) -> UiHostMeasurementCollectionInput<'_> {
        UiHostMeasurementCollectionInput {
            identity: self.identity,
            evidence_family: self.evidence_family,
            need: self.need,
            capability_report,
            evidence_generation: self.evidence_generation,
            normalization_context: self.normalization_context,
        }
    }
}

#[cfg(test)]
mod tests;
