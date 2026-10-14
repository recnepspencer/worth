//! Move-only fixed operation custody and native progress outcomes.
use super::descriptor::WorthQueryOperationDescription;
use super::*;

pub struct WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) installed_action: &'runtime worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    pub(super) runtime: &'runtime WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) description: WorthQueryOperationDescription<Schema, Intent>,
    pub(super) state: FixedState<Schema, Intent, Program, Root>,
    pub(super) initial_cause: Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) recovered: Option<worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed>,
    pub(super) result: Option<<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result>,
    pub(super) cleanup: Vec<worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) action: std::marker::PhantomData<Action>,
}

pub(super) enum FixedState<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
{
    Initial,
    Unpublished(
        WorthQueryUnpublishedRequiredApplicationMutation<
            Schema,
            Binding<Schema, Intent>,
            Program,
            Root,
        >,
    ),
    Retention(
        WorthQueryRequiredOutputRetentionFailure<Schema, Binding<Schema, Intent>, Program, Root>,
    ),
    Preparation(
        WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding<Schema, Intent>>,
    ),
    Outputs(WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>),
    Blocked(
        super::super::mutation::WorthQueryBlockedProgramSource<
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Denial,
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result,
            Demand<Schema, Root>,
        >,
    ),
    NotPerformed(
        WorthQueryApplicationMutationOutcome<
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Denial,
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result,
        >,
    ),
}

/// Each progress describes a genuine native handoff or native no-effect outcome.
pub enum WorthQueryOperationProgress {
    SourceUnpublished,
    SourcePerformed,
    SourceRetentionDenied,
    Recovery(WorthQueryProgramSourceRecoveryProgress),
    SourcePromoted,
    OutputsStarted,
    OutputsPending,
}

pub enum WorthQueryOperationAdvance<Operation, Settled, Terminal> {
    Progress {
        operation: Operation,
        progress: WorthQueryOperationProgress,
    },
    Refused {
        operation: Operation,
        denial: WorthQueryOperationAdvanceDenial,
    },
    Terminal {
        operation: Terminal,
        denial: WorthQueryOperationAdvanceDenial,
    },
    Settled(Settled),
}

#[derive(Debug)]
pub enum WorthQueryOperationAdvanceDenial {
    FreshRequestMismatch,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Mutation(WorthQueryPerformedMutationExecutionDenial),
    Recovery(WorthQueryApplicationRecoveryRequestDenial),
    Outputs(WorthQueryRequiredOutputPreparationDenial),
    Observation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Program(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial),
    Action(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    NativeTerminal,
    RecoveryCarrierUnavailable,
}
