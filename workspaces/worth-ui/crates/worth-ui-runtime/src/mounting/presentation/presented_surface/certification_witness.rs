//! Witnesses for tests and certification evidence, minted through the same
//! boundary a host uses.
//!
//! Evidence that needs displayed truth gets it the way production does: the runtime
//! claims its live lease, issues work for one surface, a host acknowledges that
//! work, and the runtime admits the acknowledgement. Nothing here constructs a
//! witness or a displayed basis directly.

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationCostReport,
    UiHostSurfacePresentationMode, UiMountedCompletedEffects, UiMountedEffectFamily,
    UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView, UiMountedFrameIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationWorkView,
    UiMountedSurfaceBindingRequirement, UiMountedSurfacePresentationCompletion,
    UiPresentationDeadline, UiSemanticSurfaceIdentity, WorthUiHostCapabilityObservationGeneration,
};

use super::{UiIssuedSurfacePresentation, UiPresentedSurfaceWitness};

/// The effects a certification host performs for the record-only work it is issued.
pub(super) const CERTIFICATION_EFFECTS: [UiMountedEffectFamily; 1] =
    [UiMountedEffectFamily::RecordedProjection];

/// A witness that `basis` reached the screen for a freshly minted semantic surface.
#[expect(
    clippy::disallowed_methods,
    reason = "certification acknowledges its issued work the way a host does"
)]
pub(crate) fn presented_surface_witness_for_certification(
    basis: UiHostObservationPresentationBasis,
) -> UiPresentedSurfaceWitness {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().expect("semantic surface");
    let witness = acknowledged_for_certification(surface, basis, |view| {
        view.acknowledge_presented(
            UiHostSurfacePresentationMode::RecordOnly,
            basis.epoch(),
            UiMountedCompletedEffects::new(CERTIFICATION_EFFECTS.to_vec()),
            UiHostPresentationCostReport::default(),
        )
    })
    .admit(basis.frame(), &CERTIFICATION_EFFECTS)
    .unwrap_or_else(|_| panic!("the runtime admits its own issued work"));
    assert_eq!(witness.displayed_basis().basis(), basis);
    witness
}

/// Work one runtime session issued for one surface, and a host's acknowledgement.
///
/// The session and its lease stay alive with the acknowledgement, so admission
/// sees the same live lease the work was issued under.
pub(super) struct UiAcknowledgedCertificationWork {
    pub(super) authority: crate::host::adapter::UiHostAdapterSessionAuthority,
    pub(super) lease: crate::mounting::presentation::UiMountedPresentationLease,
    pub(super) attempt: UiMountedPresentationAttemptIdentity,
    pub(super) requirement: UiMountedSurfaceBindingRequirement,
    pub(super) completion: UiMountedSurfacePresentationCompletion,
}

impl UiAcknowledgedCertificationWork {
    /// Admit the acknowledgement against the work this session issued, at `frame`.
    pub(super) fn admit(
        self,
        frame: UiMountedFrameIdentity,
        expected_effects: &[UiMountedEffectFamily],
    ) -> Result<UiPresentedSurfaceWitness, super::UiRefusedSurfaceAcknowledgement> {
        let Self {
            authority,
            lease: _lease,
            attempt,
            requirement,
            completion,
        } = self;
        UiPresentedSurfaceWitness::admit(
            completion,
            &authority,
            UiIssuedSurfacePresentation {
                attempt,
                requirement,
                frame,
                expected_effects,
            },
        )
    }
}

/// Issue record-only work for `surface` at `basis` under a fresh runtime session,
/// and let `acknowledge` answer it the way a host would.
pub(super) fn acknowledged_for_certification(
    surface: UiSemanticSurfaceIdentity,
    basis: UiHostObservationPresentationBasis,
    acknowledge: impl FnOnce(
        &UiMountedFrameConsumptionView<'_>,
    ) -> UiMountedSurfacePresentationCompletion,
) -> UiAcknowledgedCertificationWork {
    let projection = crate::certification_support::empty_projection_at_for_certification(
        basis.frame(),
        surface,
        basis.binding(),
    );
    let requirement = record_only_requirement(surface, basis);
    let authority = crate::host::adapter::UiHostAdapterSessionAuthority::activate(1);
    let lease = authority
        .claim_mounted_presentation_lease()
        .expect("an isolated certification session has no bound presentation lease");
    let work =
        crate::mounting::presentation::work_producer::UiMountedPresentationState::from_projection(
            &projection,
            requirement,
            None,
        )
        .issue_initial(&lease, &projection);
    let initial = work
        .into_initial_mechanics()
        .expect("the initial producer emits initial mechanics");
    let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().expect("attempt");
    let view =
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: lease.mechanics_authority(),
            host_session_identity: authority.host_session_identity(),
            protocol: current_protocol(),
            capability_generation: requirement.capability_generation(),
            capability_profile_digest: requirement.capability_profile_digest(),
            attempt,
            deadline: UiPresentationDeadline::at_tick(1),
            requirement,
            presentation_work: UiMountedPresentationWorkView::Initial(&initial),
            appearance_work: None,
            qualified_text: &NoQualifiedText,
            text_raster_work: None,
        });
    assert!(authority.admits_mounted_presentation(&view));
    let completion = acknowledge(&view);
    UiAcknowledgedCertificationWork {
        authority,
        lease,
        attempt,
        requirement,
        completion,
    }
}

fn record_only_requirement(
    surface: UiSemanticSurfaceIdentity,
    basis: UiHostObservationPresentationBasis,
) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::new(
        surface,
        basis.host_surface(),
        basis.binding(),
        WorthUiHostCapabilityObservationGeneration::new(1),
        1,
        UiHostSurfacePresentationMode::RecordOnly,
    )
}

fn current_protocol() -> worth_ui_host_contract::UiHostProtocolAgreement {
    let worth_ui_host_contract::UiHostProtocolNegotiation::Compatible(protocol) =
        worth_ui_host_contract::UiHostProtocolContract::current().negotiate()
    else {
        unreachable!("current protocol negotiates with itself")
    };
    protocol
}

struct NoQualifiedText;

impl worth_ui_host_contract::UiMountedQualifiedTextResolver for NoQualifiedText {
    fn resolve(
        &self,
        _identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<worth_ui_host_contract::UiQualifiedTextLayoutView<'_>> {
        None
    }
}
