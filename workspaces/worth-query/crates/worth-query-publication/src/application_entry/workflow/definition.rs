//! Definition authoring entries: publication and retirement are peers that
//! share the capability-mutation request shape.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
};

mod publication;
mod retirement;

pub use publication::{
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    WorthQueryWorkflowDefinitionPublicationPreparationDenialKind,
    WorthQueryWorkflowDefinitionPublicationRequest,
};
pub use retirement::{
    WorthQueryWorkflowDefinitionRetirementPreparationDenial,
    WorthQueryWorkflowDefinitionRetirementPreparationDenialKind,
    WorthQueryWorkflowDefinitionRetirementRequest,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;
