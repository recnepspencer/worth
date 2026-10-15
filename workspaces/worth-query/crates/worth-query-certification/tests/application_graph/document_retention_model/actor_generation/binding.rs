use super::declaration::*;
use super::super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::worth_query_mutation_binding;

fn resource(input: &ActorLifecycleInput) -> String { input.identity.clone() }
worth_query_mutation_binding!(
    pub ActorLifecycleBinding for ActorLifecycleInput, schema DocumentRetentionSchema,
    identity "worth.query.certification.actor-lifecycle-binding.v1", input ActorLifecycleInputBinding,
    operation ActorLifecycle, result ActorLifecycleChangedBinding,
    idempotency u64, identity "worth.query.certification.actor-lifecycle-command.v1",
    decision super::handler::Targets, denial ActorLifecycleDenialBinding,
    handler identity "worth.query.certification.actor-lifecycle-handler.v1",
    program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Document, DocumentFacts, DocumentIdentityField, String, ReadOnly, NoApplicationUnit,
    field DocumentIdentityField::reference(), value resource,
    candidates creates 1, deletes 1, links 10, unlinks 10, writes 2, emits 0,
    resources retained_representation_bytes 8192
);
#[derive(Clone)]
pub struct ActorLifecycleIntent { pub input: ActorLifecycleInput }
impl ApplicationMutationIntent<DocumentRetentionSchema> for ActorLifecycleIntent {
    type Binding = ActorLifecycleBinding;
    fn input(&self) -> &ActorLifecycleInput { &self.input }
    fn scope_binding(&self) -> <ActorLifecycleBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(DocumentIdentityField::reference(), self.input.identity.clone())
    }
}
