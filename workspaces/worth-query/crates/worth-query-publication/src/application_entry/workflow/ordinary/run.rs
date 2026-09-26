use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_advance::{
        PerformedWorkflowTransition, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::WorthQueryApplicationMutationRequest, WorthQueryWorkflowAdvancePreparationDenial,
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
type MutationKey<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::IdempotencyKey;

/// A finite set of caller-owned keys is the run budget and preserves retry meaning.
pub struct WorthQueryOrdinaryWorkflowRun<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>,
    workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec, Program>,
    instance: PublishedWorkflowInstanceRef,
}

pub struct WorthQueryOrdinaryWorkflowRunWithKeys<
    'application,
    'principal,
    'scope,
    'keys,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    run: WorthQueryOrdinaryWorkflowRun<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    >,
    keys: &'keys [MutationKey<Schema, Intent>],
}

#[derive(Debug)]
pub enum WorthQueryOrdinaryWorkflowRunStop {
    Terminal,
    /// The supplied key sequence ended; the instance may still be live.
    CallerKeysExhausted,
    /// This invocation reached the installed step ceiling, not a terminal proof.
    InstalledStepLimit,
    Interrupted(WorthQueryRequestInterruption),
    PreparationDenied(WorthQueryWorkflowAdvancePreparationDenial),
    Outcome(WorkflowProgressOutcome),
}

#[derive(Debug)]
pub struct WorthQueryOrdinaryWorkflowRunProgress {
    transitions: Vec<PerformedWorkflowTransition>,
    attempted_steps: usize,
    stop: WorthQueryOrdinaryWorkflowRunStop,
}

impl WorthQueryOrdinaryWorkflowRunProgress {
    /// Committed transitions and exact replays, in attempted-key order.
    pub fn transitions(&self) -> &[PerformedWorkflowTransition] {
        &self.transitions
    }

    /// Includes the final key that returned a wait or denial without committing.
    pub const fn attempted_steps(&self) -> usize {
        self.attempted_steps
    }

    pub const fn stop(&self) -> &WorthQueryOrdinaryWorkflowRunStop {
        &self.stop
    }
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub fn run_workflow<Spec, Program>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec, Program>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> WorthQueryOrdinaryWorkflowRun<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        WorthQueryOrdinaryWorkflowRun {
            request: self,
            workflow,
            instance,
        }
    }
}

impl<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowRun<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// Each attempted advance uses the next caller-owned key. A separate
    /// assessment, operation or approval action needs its own intent key.
    pub fn idempotency_keys<'keys>(
        self,
        keys: &'keys [MutationKey<Schema, Intent>],
    ) -> WorthQueryOrdinaryWorkflowRunWithKeys<
        'application,
        'principal,
        'scope,
        'keys,
        Schema,
        Intent,
        Spec,
        Program,
    > {
        WorthQueryOrdinaryWorkflowRunWithKeys { run: self, keys }
    }
}

impl<Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowRunWithKeys<'_, '_, '_, '_, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>
        + ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    MutationOperation<Schema, Intent>: 'static,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn execute(self) -> WorthQueryOrdinaryWorkflowRunProgress {
        let Self { run, keys } = self;
        let WorthQueryOrdinaryWorkflowRun {
            request,
            workflow,
            instance,
        } = run;
        let maximum = workflow
            .workflow_spec()
            .resources()
            .maximum_retained_transitions_per_instance() as usize;
        let mut transitions = Vec::new();
        let mut attempted_steps = 0;
        for key in keys.iter().take(maximum) {
            if let Some(interruption) = request.request_scope().interruption() {
                return WorthQueryOrdinaryWorkflowRunProgress {
                    transitions,
                    attempted_steps,
                    stop: WorthQueryOrdinaryWorkflowRunStop::Interrupted(interruption),
                };
            }
            let step = request
                .repeat_for_workflow_run()
                .without_source()
                .idempotency(key)
                .prepare_workflow_advance(workflow, instance.clone());
            attempted_steps += 1;
            let outcome = match step {
                Ok(step) => step.execute(),
                Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(actor)) => {
                    return WorthQueryOrdinaryWorkflowRunProgress {
                        transitions,
                        attempted_steps,
                        stop: WorthQueryOrdinaryWorkflowRunStop::Outcome(
                            WorkflowProgressOutcome::AwaitingActor(actor),
                        ),
                    }
                }
                Err(denial) => {
                    return WorthQueryOrdinaryWorkflowRunProgress {
                        transitions,
                        attempted_steps,
                        stop: WorthQueryOrdinaryWorkflowRunStop::PreparationDenied(denial),
                    }
                }
            };
            match outcome {
                WorkflowProgressOutcome::Completed(performed) => {
                    let terminal = performed.terminal();
                    transitions.push(performed);
                    if terminal {
                        return WorthQueryOrdinaryWorkflowRunProgress {
                            transitions,
                            attempted_steps,
                            stop: WorthQueryOrdinaryWorkflowRunStop::Terminal,
                        };
                    }
                }
                other => {
                    return WorthQueryOrdinaryWorkflowRunProgress {
                        transitions,
                        attempted_steps,
                        stop: WorthQueryOrdinaryWorkflowRunStop::Outcome(other),
                    }
                }
            }
        }
        WorthQueryOrdinaryWorkflowRunProgress {
            transitions,
            attempted_steps,
            stop: if keys.len() >= maximum {
                WorthQueryOrdinaryWorkflowRunStop::InstalledStepLimit
            } else {
                WorthQueryOrdinaryWorkflowRunStop::CallerKeysExhausted
            },
        }
    }
}
