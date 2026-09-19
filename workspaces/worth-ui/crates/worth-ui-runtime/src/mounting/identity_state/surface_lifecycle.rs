use worth_ui_host_contract::{
    UiHostSurfaceBaselineIdentity, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiHostSurfaceRegistrationDenial, UiHostSurfaceRegistrationInput,
    UiHostSurfaceRegistrationRequest, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityReport,
};

use super::{SurfaceBindingRecord, UiMountedIdentityState};
use crate::mounting::identity_view::UiSurfaceBindingConstruction;
use crate::mounting::{
    UiMountedIdentityDenial, UiSurfaceBindingIdentityView, UiSurfaceBindingProfile,
};

pub(crate) struct UiMountedSurfaceRegistrationCandidate {
    request: UiHostSurfaceRegistrationRequest,
    profile: UiSurfaceBindingProfile,
    successor_binding_revision: u64,
}

pub(crate) struct UiMountedSurfaceDeregistrationCandidate {
    semantic_surface: UiSemanticSurfaceIdentity,
    record: SurfaceBindingRecord,
    preserve_published_frame: bool,
    successor_binding_revision: u64,
}

impl UiMountedIdentityState {
    pub(crate) fn surface_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<crate::mounting::UiSurfaceBindingIdentityView> {
        self.bindings
            .values()
            .find(|record| record.view.binding_generation() == binding)
            .map(|record| record.view)
    }

    pub(crate) fn prepare_surface_registration(
        &self,
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &WorthUiHostCapabilityReport,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiMountedSurfaceRegistrationCandidate, UiMountedIdentityDenial> {
        self.prepare_surface_registration_with_identity(
            protocol,
            capability_report,
            semantic_surface,
            mode,
            profile,
            None,
        )
    }

    pub(crate) fn prepare_surface_rebind_registration(
        &self,
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &WorthUiHostCapabilityReport,
        semantic_surface: UiSemanticSurfaceIdentity,
        host_surface: UiHostSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiMountedSurfaceRegistrationCandidate, UiMountedIdentityDenial> {
        self.prepare_surface_registration_with_identity(
            protocol,
            capability_report,
            semantic_surface,
            mode,
            profile,
            Some(host_surface),
        )
    }

    fn prepare_surface_registration_with_identity(
        &self,
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &WorthUiHostCapabilityReport,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
        retained_host_surface: Option<UiHostSurfaceIdentity>,
    ) -> Result<UiMountedSurfaceRegistrationCandidate, UiMountedIdentityDenial> {
        self.require_surface(semantic_surface)?;
        if self.bindings.contains_key(&semantic_surface) {
            return Err(UiMountedIdentityDenial::SurfaceAlreadyBound);
        }
        let host_surface = retained_host_surface
            .map_or_else(UiHostSurfaceIdentity::mint_unbound, Ok)
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let binding_generation = UiSurfaceBindingGeneration::mint_unbound()
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let successor_binding_revision = super::next(&super::NEXT_STATE_REVISION)?;
        let request =
            UiHostSurfaceRegistrationRequest::from_runtime(UiHostSurfaceRegistrationInput {
                host_session_identity: self.host_session_identity.as_u64(),
                semantic_surface_identity: semantic_surface,
                host_surface_identity: host_surface,
                binding_generation,
                protocol,
                capability_generation: capability_report.observation_generation(),
                capability_profile_digest: capability_report.profile_identity_digest(),
                presentation_mode: mode,
            });
        Ok(UiMountedSurfaceRegistrationCandidate {
            request,
            profile,
            successor_binding_revision,
        })
    }

    pub(crate) fn commit_surface_registration(
        &mut self,
        candidate: UiMountedSurfaceRegistrationCandidate,
        baseline: UiHostSurfaceBaselineIdentity,
    ) -> UiSurfaceBindingIdentityView {
        let request = candidate.request;
        debug_assert_eq!(baseline.binding_generation(), request.binding_generation());
        debug_assert!(!self
            .bindings
            .contains_key(&request.semantic_surface_identity()));
        let view = UiSurfaceBindingIdentityView::new(UiSurfaceBindingConstruction {
            request,
            binding_generation: request.binding_generation(),
            profile: candidate.profile,
            baseline,
        });
        self.bindings.insert(
            request.semantic_surface_identity(),
            SurfaceBindingRecord { view, request },
        );
        self.pending_projection_changes
            .mark_changed_surface(request.semantic_surface_identity());
        self.binding_revision = candidate.successor_binding_revision;
        view
    }

    pub(crate) fn prepare_surface_deregistration(
        &self,
        binding: UiSurfaceBindingGeneration,
        preserve_published_frame: bool,
    ) -> Result<UiMountedSurfaceDeregistrationCandidate, UiMountedIdentityDenial> {
        let (semantic_surface, record) = self
            .bindings
            .iter()
            .find(|(_, record)| record.view.binding_generation() == binding)
            .map(|(surface, record)| (*surface, *record))
            .ok_or(UiMountedIdentityDenial::UnknownSurfaceBinding)?;
        let successor_binding_revision = super::next(&super::NEXT_STATE_REVISION)?;
        Ok(UiMountedSurfaceDeregistrationCandidate {
            semantic_surface,
            record,
            preserve_published_frame,
            successor_binding_revision,
        })
    }

    pub(crate) fn commit_surface_deregistration(
        &mut self,
        candidate: UiMountedSurfaceDeregistrationCandidate,
    ) -> UiSemanticSurfaceIdentity {
        let removed = self.bindings.remove(&candidate.semantic_surface);
        debug_assert!(removed.is_some_and(|record| record.request == candidate.record.request));
        if !candidate.preserve_published_frame {
            self.unprojected_semantic_predecessor = self.semantic_predecessor().cloned();
            self.unprojected_pointer_predecessor = self
                .pointer_predecessor()
                .map(|state| state.after_surface_deregistration(candidate.semantic_surface));
            self.unprojected_appearance_predecessor =
                self.appearance_predecessor().map(|appearance| {
                    appearance.after_surface_deregistration(candidate.semantic_surface)
                });
            self.current_frame = None;
            self.current_receipt_basis = None;
            self.current_projection = None;
            self.current_manifest = None;
            self.current_core = None;
            self.current_publication = None;
            self.current_reuse_contract = None;
        }
        self.pending_projection_changes
            .mark_removed_surface(candidate.semantic_surface);
        self.binding_revision = candidate.successor_binding_revision;
        candidate.semantic_surface
    }
}

impl UiMountedSurfaceRegistrationCandidate {
    pub(crate) fn request(&self) -> UiHostSurfaceRegistrationRequest {
        self.request
    }
}

impl UiMountedSurfaceDeregistrationCandidate {
    pub(crate) fn request(&self) -> UiHostSurfaceRegistrationRequest {
        self.record.request
    }
}

pub(crate) fn map_registration_denial(
    denial: UiHostSurfaceRegistrationDenial,
) -> UiMountedIdentityDenial {
    match denial {
        UiHostSurfaceRegistrationDenial::Unsupported => {
            UiMountedIdentityDenial::SurfaceRegistrationUnsupported
        }
        UiHostSurfaceRegistrationDenial::KnownEmptyBaselineUnavailable => {
            UiMountedIdentityDenial::KnownEmptyBaselineUnavailable
        }
        UiHostSurfaceRegistrationDenial::ForeignRegistration => {
            UiMountedIdentityDenial::ForeignSurfaceRegistration
        }
        UiHostSurfaceRegistrationDenial::CapacityExceeded => {
            UiMountedIdentityDenial::HostSurfaceCapacityExceeded
        }
    }
}
