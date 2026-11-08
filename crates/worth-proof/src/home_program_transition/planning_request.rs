//! A Home-owner request before a plan exists; never step execution permission.
use super::*;

#[derive(Debug)]
/// Planning permission cannot stand in for an admitted execution step.
/// ```compile_fail,E0308
/// use worth_proof::{AdmittedHomeProgramTransitionPlanningRequest, AdmittedHomeProgramTransitionStep};
/// fn no_step(request: AdmittedHomeProgramTransitionPlanningRequest<()>) -> AdmittedHomeProgramTransitionStep<()> { request }
/// ```
pub struct AdmittedHomeProgramTransitionPlanningRequest<Source> {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    source: Source,
}
impl<Source> AdmittedHomeProgramTransitionPlanningRequest<Source> {
    pub fn source(&self) -> &Source {
        &self.source
    }
}
impl HomeProgramTransitionOwner {
    pub fn admit_planning_request<Source>(
        &self,
        source: Source,
    ) -> AdmittedHomeProgramTransitionPlanningRequest<Source> {
        AdmittedHomeProgramTransitionPlanningRequest {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            source,
        }
    }
}
impl HomeProgramTransitionAuthority {
    /// Admit the original owner/fence before inspecting the source's scope.
    /// The actual receiver separately verifies the exact source admission.
    pub fn admit_planning_request<Source>(
        &self,
        request: &AdmittedHomeProgramTransitionPlanningRequest<Source>,
        source_home: [u8; 16],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit_owner(&request.occurrence, request.home, request.incarnation)?;
        if source_home != self.home {
            return Err(HomeProgramTransitionAdmissionDenial::SourceMismatch);
        }
        Ok(())
    }
}
