//! Owning action composition delegates the native mutation and output owners.
mod close;
mod construction;
mod descriptor;
mod fixed;
mod fixed_advance;
mod fixed_outputs;
mod fixed_publication;
mod fixed_recovery;
mod request_view;
mod settled;
mod terminal;
pub use close::{WorthQueryClosedOperation, WorthQueryOperationCloseDenial};
pub use construction::WorthQueryOperationConstructionDenial;
pub use descriptor::WorthQueryOperationDescription;
pub use fixed::{
    WorthQueryFixedOperation, WorthQueryOperationAdvance, WorthQueryOperationAdvanceDenial,
    WorthQueryOperationProgress,
};
pub use settled::{WorthQueryOperationDeliveryDenial, WorthQuerySettledFixedOperation};
pub use terminal::WorthQueryTerminalFixedOperation;

use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationFixedAction, ApplicationFixedActionDelivery,
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
    ApplicationRequiredOutputRoot,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::application_contribution::{
    WorthQueryAdvancementPhase, WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

type Binding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type Connection<Schema, Root> =
    <<Root as ApplicationOutputGraphShape<Schema>>::RootConnection as ApplicationConnectionShape<
        Schema,
    >>::Binding;
type Demand<Schema, Root> =
    <Connection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type Source<Schema, Root> = <<Demand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootQuery<Schema, Root> = <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type Delivery<Schema, Intent, Action> =
    <Action as ApplicationFixedAction<Schema, Binding<Schema, Intent>>>::Delivery;
type Read<Schema, Intent, Action> =
    <Delivery<Schema, Intent, Action> as ApplicationFixedActionDelivery<
        Schema,
        Binding<Schema, Intent>,
    >>::Read;

type FixedRoot<Schema, Intent, Action> =
    <Delivery<Schema, Intent, Action> as ApplicationFixedActionDelivery<
        Schema,
        Binding<Schema, Intent>,
    >>::Root;

type MutationInput<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;
type MutationScope<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding;
type MutationPrincipal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity;
type SourceIntent<Schema, Root> = <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Input;
type SourceScope<Schema, Root> =
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding;
type SourcePrincipal<Schema, Root> =
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type SourceValue<Schema, Root> = <<Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> = WorthQueryOperationAdvance<
    WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
    WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
    WorthQueryTerminalFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
>;
