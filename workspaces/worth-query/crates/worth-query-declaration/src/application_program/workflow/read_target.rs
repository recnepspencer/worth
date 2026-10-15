//! Explicit inert target for definition-wide disclosure; no subject omission widening.
use super::{ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowSpec};
use std::marker::PhantomData;
/// Selects exactly one definition under the request's branch and this Spec.
/// Enumeration covers its instances across subjects; it implies no tenant relationship.
pub struct ApplicationWorkflowDefinitionReadTarget<Spec: ApplicationWorkflowSpec> {
    identity: ApplicationWorkflowDefinitionIdentity,
    marker: PhantomData<fn() -> Spec>,
}
impl<Spec: ApplicationWorkflowSpec> ApplicationWorkflowDefinitionReadTarget<Spec> {
    pub fn new(identity: ApplicationWorkflowDefinitionIdentity) -> Self {
        Self {
            identity,
            marker: PhantomData,
        }
    }
    pub fn identity(&self) -> &ApplicationWorkflowDefinitionIdentity {
        &self.identity
    }
}
