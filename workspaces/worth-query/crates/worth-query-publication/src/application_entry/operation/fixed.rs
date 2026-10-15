//! Move-only fixed operation custody and native progress outcomes.
use super::descriptor::WorthQueryOperationDescription;
use super::*;

/// Owns the original fixed-output description and native source, output and cleanup custody.
/// Construction retains the authentic runtime allocation and complete installed action.
/// Each advancement borrows fresh request authority; refusals return the owning operation.
#[must_use = "retain native publication/output custody until settlement or terminal handoff"]
pub struct WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) installed_action: worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    pub(super) description: WorthQueryOperationDescription<Schema, Intent>,
    pub(super) state: FixedState<Schema, Intent, Program, Root>,
    pub(super) initial_cause: Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) recovered: Option<worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed>,
    pub(super) result: Option<<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result>,
    pub(super) cleanup: WorthQueryOperationCleanup,
    pub(super) action: std::marker::PhantomData<Action>,
    // Native carriers above release while the authentic allocation remains retained.
    pub(super) runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
}

impl<Schema, Intent, Program, Root, Action>
    WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    /// The original owner-issued branch selector; this grants no authority.
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.description.branch
    }
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
    Replay(worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt),
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
    /// The original key resolved to committed evidence; no handler ran.
    SourceReplayObserved,
    /// Native registered required-output custody resumed.
    RegisteredOutputsRecovered,
    SourceRetentionDenied,
    Recovery(WorthQueryProgramSourceRecoveryProgress),
    SourcePromoted,
    OutputsStarted,
    OutputsPending,
}

/// Returns native progress, refused custody, terminal custody, or settled output ownership.
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
/// Explains a refused or terminal advancement without releasing its native obligations.
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
