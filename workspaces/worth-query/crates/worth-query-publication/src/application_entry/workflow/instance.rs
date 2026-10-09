use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_instance::{
    PreparedWorkflowInstanceStart, PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome,
    WorthQueryWorkflowInstanceAdapter,
};
use worth_query_installation::facade::ApplicationSchema;

mod cancellation;

pub use cancellation::WorthQueryWorkflowInstanceCancellationRequest;

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
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

/// The kind of a `WorthQueryWorkflowInstancePreparationDenial`.
impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_instance_start<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.application_runtime();
        runtime.with_application_advancement(&request_scope, |phase| {
                self.prepare_workflow_instance_start_in_advancement(&phase ,workflow, definition)
            }).map_err(|cause| WorthQueryWorkflowInstancePreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ))?
    }

    pub(in crate::application_entry) fn prepare_workflow_instance_start_in_advancement<'workflow, Spec>(
        self, _phase: &AdvancementPhase<'_>,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {

        self.prepare_start(workflow, |selected, installed, key, admission| {
            WorthQueryWorkflowInstanceAdapter::prepare::<
                Schema,
                <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
                Spec,
            >(selected, installed, definition, key, admission)
        })


    }

    /// Ends `instance` and continues its work as a new instance on the
    /// current `target` definition, beginning at the node `resume_at` names.
    ///
    /// The successor carries only performed effects, as history it can never
    /// repeat. Proposals, evidence and approvals are re-established by running
    /// the nodes that produce them, so each one the successor consumes must
    /// run before its consumer. An approval whose operation has not yet run
    /// blocks migration until it settles under the source. The start
    /// capability authorizes migration, and with it ending the source.
    pub fn prepare_workflow_instance_migration<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.application_runtime();
        runtime.with_application_advancement(&request_scope, |_phase| {

        self.prepare_start(workflow, |selected, installed, key, admission| {
            WorthQueryWorkflowInstanceAdapter::prepare_migration::<
                Schema,
                <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
                Spec,
            >(selected, installed, instance, target, resume_at, key, admission)
        })

        }).map_err(|cause| WorthQueryWorkflowInstancePreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ))?
    }

    /// Continues a fork's copy of `instance`, started on another branch, as a
    /// new instance on the fork's current `target` definition, beginning at
    /// the node `resume_at` names. Issue it on the fork.
    ///
    /// A fork copies history, not execution: the copy's approvals and
    /// receipts open nothing. The continuation obeys the migration law and
    /// ends only the fork's copy; the instance on its own branch is
    /// untouched. `target` may be the copied definition itself, named by the
    /// reference issued on the branch the fork was taken from or by its
    /// `held_on(fork)` reference, while the fork holds it current.
    pub fn prepare_workflow_fork_continuation<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.application_runtime();
        runtime.with_application_advancement(&request_scope, |_phase| {

        self.prepare_start(workflow, |selected, installed, key, admission| {
            WorthQueryWorkflowInstanceAdapter::prepare_fork_continuation::<
                Schema,
                <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
                Spec,
            >(selected, installed, instance, target, resume_at, key, admission)
        })

        }).map_err(|cause| WorthQueryWorkflowInstancePreparationDenial::RequestAdmission(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }

    #[allow(clippy::type_complexity)]
    fn prepare_start<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        prepare: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                '_,
                Schema,
            >,
            &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
                Schema,
                Spec,>,
            [u8; 32],
            worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation<
                Schema,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
            >,
        ) -> Result<
            PreparedWorkflowInstanceStart<
                Schema,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
            >,
            WorkflowInstancePreparationDenial,
        >,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let (application, prepared, idempotency) = self.prepare_instance(workflow, prepare)?;
        Ok(WorthQueryWorkflowInstanceStartRequest {
            application,
            prepared,
            idempotency,
        })
    }

    /// Admits the request on its selected branch and prepares one instance
    /// lifecycle action with its key.
    #[allow(clippy::type_complexity)]
    fn prepare_instance<'workflow, Spec, Prepared>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        prepare: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                '_,
                Schema,
            >,
            &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
                Schema,
                Spec,>,
            [u8; 32],
            worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation<
                Schema,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
            >,
        ) -> Result<Prepared, WorkflowInstancePreparationDenial>,
    ) -> Result<
        (
            &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            Prepared,
            worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
        ),
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowInstancePreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowInstancePreparationDenial::RequestAdmission)?;
        let staged = self.stage().map_err(WorthQueryWorkflowInstancePreparationDenial::RequestAdmission)?;
        let identities = self
            .identities()
            .map_err(WorthQueryWorkflowInstancePreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&self, &identities, staged, &selected)
            .map_err(WorthQueryWorkflowInstancePreparationDenial::RequestAdmission)?;
        let prepared = prepare(
            &selected,
            workflow.workflow_spec_for(&selected),
            *mutation.idempotency.key_identity(),
            mutation.admission,
        )
        .map_err(WorthQueryWorkflowInstancePreparationDenial::InstancePreparation)?;
        Ok((application, prepared, mutation.idempotency))
    }
}

/// A prepared workflow instance start, migration or fork continuation. `execute` attempts
/// its commit.
pub struct WorthQueryWorkflowInstanceStartRequest<'application, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
{
    application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
    idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowInstanceStartRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowInstanceStartOutcome {
        self.application
            .with_application_advancement(&self.prepared.request_scope().clone(), |phase| {
                self.execute_in_advancement(&phase)
            })
            .unwrap_or_else(|cause| {
                WorkflowInstanceStartOutcome::Application(
                    cause
                        .into_commit_outcome()
                        .landed()
                        .expect_err("request admission cannot commit"),
                )
            })
    }

    pub(in crate::application_entry) fn execute_in_advancement(
        self,
        phase: &AdvancementPhase<'_>,
    ) -> WorkflowInstanceStartOutcome {
        WorthQueryWorkflowInstanceAdapter::compare_and_commit(
            phase,
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}

mod preparation_denial;
pub use preparation_denial::{
    WorthQueryWorkflowInstancePreparationDenial, WorthQueryWorkflowInstancePreparationDenialKind,
};
