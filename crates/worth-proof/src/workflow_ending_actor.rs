//! Ending-record provenance keeps its concrete owner-sealed actor type.

/// An ending-record witness over the actual actor admitted by its runtime.
/// Sinks require their concrete sealed actor type, never public provenance DATA.
/// This carrier grants no operation, branch or later request authority.
///
/// A public DATA value cannot replace a sink's owner-sealed actor:
/// ```compile_fail,E0308
/// use worth_proof::AdmittedWorkflowEndingActor;
/// mod owner {
///     pub struct AdmittedActor(());
///     pub fn record(_: AdmittedWorkflowEndingActor<AdmittedActor>) {}
/// }
/// owner::record(AdmittedWorkflowEndingActor::from_admitted_actor(42u64));
/// ```
#[derive(Clone, Copy)]
pub struct AdmittedWorkflowEndingActor<Actor> {
    actor: Actor,
}
impl<Actor> AdmittedWorkflowEndingActor<Actor> {
    /// Retain the exact sealed actor supplied by its checked owner path.
    /// The receiving sink must name that concrete actor type. Instantiating
    /// this with arbitrary DATA does not produce a witness of a sealed type.
    pub fn from_admitted_actor(actor: Actor) -> Self {
        Self { actor }
    }
    pub fn actor(&self) -> &Actor {
        &self.actor
    }
}
