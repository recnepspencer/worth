#[cfg(any(test, feature = "certification-support"))]
#[path = "presented_surface/certification_witness.rs"]
mod certification_witness;
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use certification_witness::presented_surface_witness_for_certification;
#[cfg(test)]
#[path = "presented_surface/admission_tests.rs"]
mod admission_tests;

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountedEffectFamily, UiMountedFrameIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiMountedSurfacePresentationCompletion,
};

/// One surface's host acknowledgement, admitted against the work this runtime issued.
///
/// A host mints the completion only by acknowledging runtime-issued work. Admission
/// proves the acknowledgement answers this runtime's live lease and the exact attempt,
/// surface binding, and frame issued, and that its mode and effects meet the work's
/// requirement. Everything that commits displayed truth consumes this witness rather
/// than a bare presentation basis.
pub(crate) struct UiPresentedSurfaceWitness {
    completion: UiMountedSurfacePresentationCompletion,
}

/// The surface, frame, binding, and epoch a host acknowledgement proved on screen.
///
/// Only an admitted [`UiPresentedSurfaceWitness`] creates one, so holding it is
/// holding displayed truth for exactly that surface generation. A basis a host
/// reports on an observation only selects which retained record to read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiDisplayedSurfaceBasis {
    basis: UiHostObservationPresentationBasis,
}

/// An acknowledgement admission refused. It proves nothing on screen; only the
/// adapter work the host reports spending survives, as cost evidence.
pub(in crate::mounting::presentation) struct UiRefusedSurfaceAcknowledgement {
    cost: Box<worth_ui_host_contract::UiHostPresentationCostReport>,
}

impl UiRefusedSurfaceAcknowledgement {
    pub(in crate::mounting::presentation) fn cost(
        &self,
    ) -> worth_ui_host_contract::UiHostPresentationCostReport {
        *self.cost
    }
}

/// The work one surface was issued, which its acknowledgement must answer.
pub(in crate::mounting::presentation) struct UiIssuedSurfacePresentation<'a> {
    pub(in crate::mounting::presentation) attempt: UiMountedPresentationAttemptIdentity,
    pub(in crate::mounting::presentation) requirement: UiMountedSurfaceBindingRequirement,
    pub(in crate::mounting::presentation) frame: UiMountedFrameIdentity,
    pub(in crate::mounting::presentation) expected_effects: &'a [UiMountedEffectFamily],
}

impl UiPresentedSurfaceWitness {
    pub(in crate::mounting::presentation) fn admit(
        completion: UiMountedSurfacePresentationCompletion,
        authority: &crate::host::adapter::UiHostAdapterSessionAuthority,
        issued: UiIssuedSurfacePresentation<'_>,
    ) -> Result<Self, UiRefusedSurfaceAcknowledgement> {
        let answers_issued_work = authority.issued_mounted_completion(&completion)
            && completion.attempt() == issued.attempt
            && completion.requirement() == issued.requirement
            && completion.frame() == issued.frame;
        let meets_requirement = completion.mode() == issued.requirement.presentation_mode()
            && super::terminal::completion_effects_satisfy(
                issued.expected_effects,
                completion.effects().families(),
                completion.cost(),
            );
        if answers_issued_work && meets_requirement {
            Ok(Self { completion })
        } else {
            Err(UiRefusedSurfaceAcknowledgement {
                cost: Box::new(completion.cost()),
            })
        }
    }

    pub(crate) fn displayed_basis(&self) -> UiDisplayedSurfaceBasis {
        UiDisplayedSurfaceBasis {
            basis: self.completion.presented_basis(),
        }
    }

    pub(crate) fn requirement(&self) -> UiMountedSurfaceBindingRequirement {
        self.completion.requirement()
    }

    pub(crate) fn cost(&self) -> worth_ui_host_contract::UiHostPresentationCostReport {
        self.completion.cost()
    }

    pub(in crate::mounting::presentation) fn into_completion(
        self,
    ) -> UiMountedSurfacePresentationCompletion {
        self.completion
    }
}

impl UiDisplayedSurfaceBasis {
    pub(crate) const fn basis(self) -> UiHostObservationPresentationBasis {
        self.basis
    }

    pub(crate) const fn host_surface(self) -> worth_ui_host_contract::UiHostSurfaceIdentity {
        self.basis.host_surface()
    }

    pub(crate) const fn binding(self) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        self.basis.binding()
    }

    pub(crate) const fn frame(self) -> UiMountedFrameIdentity {
        self.basis.frame()
    }
}
