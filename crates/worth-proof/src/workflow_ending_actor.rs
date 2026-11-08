//! Ending provenance retains the actual sealed principal or Home request.
use crate::AdmittedHomeProgramTransitionPlanningRequest;

/// The exact source admitted for a workflow ending, never portable actor DATA.
/// This grants no operation, branch or later request authority.
///
/// Public DATA cannot replace a sink's owner-sealed principal:
/// ```compile_fail,E0308
/// use worth_proof::AdmittedWorkflowEndingActor;
/// mod owner {
///     pub struct AdmittedActor(());
///     pub fn record(_: AdmittedWorkflowEndingActor<AdmittedActor, ()>) {}
/// }
/// owner::record(AdmittedWorkflowEndingActor::<u64, ()>::from_admitted_actor(42));
/// ```
/// A Home source requires the actual sealed planning request:
/// ```compile_fail,E0308
/// use worth_proof::AdmittedWorkflowEndingActor;
/// let _ = AdmittedWorkflowEndingActor::<(), ()>::from_home_program_transition(());
/// ```
pub enum AdmittedWorkflowEndingActor<PrincipalActor, HomeSource> {
    Principal(PrincipalActor),
    HomeProgramTransition(AdmittedHomeProgramTransitionPlanningRequest<HomeSource>),
}
impl<PrincipalActor, HomeSource> AdmittedWorkflowEndingActor<PrincipalActor, HomeSource> {
    /// Sinks name their concrete privately issued principal type.
    pub fn from_admitted_actor(actor: PrincipalActor) -> Self {
        Self::Principal(actor)
    }
    /// Retain the original Home request. Its receiving owner must check the
    /// attached Home, incarnation and exact source before using this witness.
    pub fn from_home_program_transition(
        request: AdmittedHomeProgramTransitionPlanningRequest<HomeSource>,
    ) -> Self {
        Self::HomeProgramTransition(request)
    }
}
