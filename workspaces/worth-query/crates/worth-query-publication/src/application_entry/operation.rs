//! Owning action composition delegates the native mutation and output owners.
mod cleanup;
mod close;
mod mode_construction;
mod source_admission;
mod source_only;
mod workflow;
pub use cleanup::{WorthQueryOperationCleanup, WorthQueryOperationCleanupReceipt};
pub use mode_construction::WorthQueryOperationModeFactory;
pub use source_admission::{WorthQuerySourceDeliveryMode, WorthQuerySourceOperationAdmission};
pub use source_only::{
    WorthQueryCompletedSourceOperation, WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
    WorthQuerySourceOnlyDenial, WorthQuerySourceOnlyOperation, WorthQuerySourceOnlyPosture,
    WorthQueryTerminalSourceOperation,
};
pub use workflow::{
    WorthQueryCompletedWorkflowOperation, WorthQueryOwnedWorkflowBinding,
    WorthQueryTerminalWorkflowOperation, WorthQueryWorkflowAcceptance,
    WorthQueryWorkflowEffectRecoveryDenial, WorthQueryWorkflowOperation,
    WorthQueryWorkflowOperationAdvance, WorthQueryWorkflowOperationDenial,
};
mod capture_refusal;
mod construction;
pub use capture_refusal::{
    WorthQueryOperationInputCaptureRefusal, WorthQueryOperationInputCaptureRetryDenial,
};
mod projection_installation;
pub use projection_installation::WorthQueryOutputProjectionInstallation;
mod batch_delivery;
mod current_delivery;
mod delivery_policy;
mod descriptor;
mod discovered;
pub use discovered::{
    WorthQueryDiscoveredOperation, WorthQuerySettledDiscoveredOperation,
    WorthQueryTerminalDiscoveredOperation,
};
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Compiler-only bodies certify move-only delivery handoffs"
)]
mod discovered_delivery_handoff_contract;
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Compiler-only bodies certify move-only intake and recovery handoffs"
)]
mod discovered_handoff_contract;
#[cfg(test)]
#[expect(dead_code, reason = "Compiler bodies prove retained construction without cloning native custody")]
mod retained_runtime_contract;
mod fixed;
mod fixed_advance;
mod fixed_close;
mod fixed_outputs;
mod fixed_publication;
mod fixed_recovery;
mod fixed_replay;
mod request_view;
mod settled;
pub use delivery_policy::WorthQueryOperationDeliveryPolicy;
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
    ApplicationConnectionShape, ApplicationAction, ApplicationFixedActionDelivery,
    ApplicationFixedQueryDelivery, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape, ApplicationRequiredOutputRoot,
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
    <Action as ApplicationAction<Schema, Binding<Schema, Intent>>>::Delivery;
type Read<Schema, Intent, Action> =
    <Delivery<Schema, Intent, Action> as ApplicationFixedQueryDelivery<
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
type FixedAdvance<Schema, Intent, Program, Root, Action> = WorthQueryOperationAdvance<
    WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>,
    WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>,
    WorthQueryTerminalFixedOperation<Schema, Intent, Program, Root, Action>,
>;

pub use worth_query_execution::publication_boundary::operation_input_capture::WorthQueryOperationInputCaptureDenial;

// Delivery derives its query identity and value from the same installed binding.
type DeliveredProjection<Schema, ReadBinding> = crate::domain_computation::WorthQueryPublishedApplicationResult<
    <ReadBinding as ApplicationQueryBinding<Schema>>::Query,
    <<ReadBinding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
>;
type DeliveredProjectionBatch<Schema, ReadBinding> = crate::domain_computation::WorthQueryPublishedApplicationQueryBatch<
    <ReadBinding as ApplicationQueryBinding<Schema>>::Query,
    <<ReadBinding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
>;
