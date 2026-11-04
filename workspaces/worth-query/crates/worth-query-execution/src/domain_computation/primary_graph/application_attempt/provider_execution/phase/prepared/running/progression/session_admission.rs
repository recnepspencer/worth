use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;

pub(super) struct WorthQueryAdmittedProviderSession<'run> {
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    mutation_run:
        crate::domain_computation::provider_session::WorthQueryProviderSessionBoundMutationRun,
}

pub(super) struct WorthQueryProviderSessionAdmissionFailure {
    outcome: WorthQueryProviderProgressionOutcome,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
}

pub(super) struct WorthQueryRegisteredProviderSession<'run> {
    registered: super::WorthQueryRegisteredProviderAttempt<'run>,
    mutation_run:
        crate::domain_computation::provider_session::WorthQueryProviderSessionBoundMutationRun,
}

pub(super) struct WorthQueryProviderSessionRegistrationFailure {
    outcome: WorthQueryProviderProgressionOutcome,
    mutation_run:
        crate::domain_computation::provider_session::WorthQueryProviderSessionBoundMutationRun,
}

pub(super) fn admit_provider_session<'run>(
    phase: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    running: &'run mut crate::domain_computation::WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    product: crate::basis::WorthQueryProductBranchLease,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
) -> Result<WorthQueryAdmittedProviderSession<'run>, WorthQueryProviderSessionAdmissionFailure> {
    admit_provider_plan(
        phase,
        running.admit_provider_execution_plan(graph),
        product,
        mutation_run,
    )
}

/// Borrowed ordinary runs and owned coupled runs enter the same protocol owner.
pub(super) fn admit_provider_plan<'run>(
    phase: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    plan: Result<
        crate::domain_computation::provider_session::WorthQueryAdmittedProviderExecutionPlan<'run>,
        crate::domain_computation::WorthQueryProviderSessionFailure,
    >,
    product: crate::basis::WorthQueryProductBranchLease,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
) -> Result<WorthQueryAdmittedProviderSession<'run>, WorthQueryProviderSessionAdmissionFailure> {
    let staged = match plan
        .and_then(|plan| plan.bind_application_product(product))
        .and_then(|plan| plan.readmit(phase))
        .and_then(|session| session.prepare())
        .map(|prepared| prepared.bind_reads_and_effects())
    {
        Ok(staged) => staged,
        Err(cause) => {
            return Err(WorthQueryProviderSessionAdmissionFailure {
                outcome: WorthQueryProviderProgressionOutcome::Denied(
                    crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                        DenialStage::ProviderPlan,format!("provider session admission: {cause:?}"))),
                mutation_run,
            })
        }
    };
    let terminal_binding = staged.provider_session_terminal_binding();
    match mutation_run.bind_provider_session(terminal_binding) {
        Ok(bound) => Ok(WorthQueryAdmittedProviderSession {
            staged,
            mutation_run: bound,
        }),
        Err(mutation_run) => {
            let _ = staged.abort();
            Err(WorthQueryProviderSessionAdmissionFailure {
                outcome: WorthQueryProviderProgressionOutcome::Denied(
                    crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                        DenialStage::ProviderPlan, "provider session does not match the admitted mutation run")),
                mutation_run,
            })
        }
    }
}

pub(super) fn admit_coupled_provider_session<'run>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    product: crate::basis::WorthQueryProductBranchLease,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
) -> CoupledProviderSessionAdmission<'run> {
    use crate::domain_computation::provider_session::CoupledProviderPlanAdmission as Plan;
    match running.into_coupled_provider_execution_plan(execution, graph) {
        Plan::Admitted { plan, returned } => CoupledProviderSessionAdmission::Entered {
            session: admit_provider_plan(phase, Ok(plan), product, mutation_run),
            returned,
        },
        Plan::Refused { cause, returned } => CoupledProviderSessionAdmission::Entered {
            session: admit_provider_plan(phase, Err(cause), product, mutation_run),
            returned,
        },
        Plan::AllocationDenied { run, cause } => {
            CoupledProviderSessionAdmission::AllocationDenied {
                run,
                cause,
                mutation_run,
            }
        }
    }
}
pub(super) enum CoupledProviderSessionAdmission<'run> {
    Entered {
        session: Result<
            WorthQueryAdmittedProviderSession<'run>,
            WorthQueryProviderSessionAdmissionFailure,
        >,
        returned: crate::domain_computation::provider_session::ReturnedProviderDirectRun,
    },
    AllocationDenied {
        run: crate::domain_computation::WorthQueryRunningDirectRun,
        cause: worth_execution::ExecutionAllocationDenial,
        mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    },
}

impl<'run> WorthQueryAdmittedProviderSession<'run> {
    #[allow(dead_code)]
    pub(super) fn terminal_binding(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.staged.provider_session_terminal_binding()
    }

    pub(super) fn register<Schema, Operation, Input, Scope>(
        self,
        authorization: &mut crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        prepared: super::WorthQueryPreparedApplicationProviderAttempt,
        attempt_basis: super::WorthQueryApplicationAttemptBasis,
        context: super::WorthQueryProviderAttemptRegistrationContext<
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryRegisteredProviderSession<'run>,
        WorthQueryProviderSessionRegistrationFailure,
    > {
        match authorization.register_provider_attempt(
            prepared,
            self.staged,
            attempt_basis,
            context,
            allocation_policy,
        ) {
            Ok(registered) => Ok(WorthQueryRegisteredProviderSession {
                registered,
                mutation_run: self.mutation_run,
            }),
            Err(outcome) => Err(WorthQueryProviderSessionRegistrationFailure {
                outcome,
                mutation_run: self.mutation_run,
            }),
        }
    }
}

#[cfg(test)]
mod registration_conflict_tests {
    include!("registration_conflict_tests.rs");
    // A child, so the court reuses these private helpers without widening them.
    mod interrupted_registration_tests {
        include!("session_admission/interrupted_registration_tests.rs");
    }
}

impl WorthQueryProviderSessionAdmissionFailure {
    pub(super) fn into_completion(self) -> super::WorthQueryProviderProgressionCompletion {
        super::WorthQueryProviderProgressionCompletion {
            outcome: self.outcome,
            cleanup: super::WorthQueryApplicationMutationCleanupOwner::Unbound(self.mutation_run),
        }
    }
}

impl WorthQueryProviderSessionRegistrationFailure {
    pub(super) fn into_completion(self) -> super::WorthQueryProviderProgressionCompletion {
        super::WorthQueryProviderProgressionCompletion {
            outcome: self.outcome,
            cleanup: super::WorthQueryApplicationMutationCleanupOwner::ProviderBound(
                self.mutation_run,
            ),
        }
    }
}

impl<'run> WorthQueryRegisteredProviderSession<'run> {
    pub(super) fn into_coupled_parts(
        self,
    ) -> (
        super::WorthQueryRegisteredProviderAttempt<'run>,
        crate::domain_computation::provider_session::WorthQueryProviderSessionBoundMutationRun,
    ) {
        (self.registered, self.mutation_run)
    }

    pub(super) fn deny(
        self,
        outcome: WorthQueryProviderProgressionOutcome,
    ) -> super::WorthQueryProviderProgressionCompletion {
        super::WorthQueryProviderProgressionCompletion {
            outcome,
            cleanup: super::WorthQueryApplicationMutationCleanupOwner::ProviderBound(
                self.mutation_run,
            ),
        }
    }

    pub(super) fn progress<Schema, Operation, Input, Scope>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        authority: &super::WorthQueryApplicationCommitProgressionAuthority<
            '_,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> super::WorthQueryProviderProgressionCompletion
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        super::WorthQueryProviderProgressionCompletion {
            outcome: self
                .registered
                .progress(execution, phase, authority, allocation_policy),
            cleanup: super::WorthQueryApplicationMutationCleanupOwner::ProviderBound(
                self.mutation_run,
            ),
        }
    }
}

#[cfg(test)]
#[path = "session_admission/overlay_conflict_tests.rs"]
mod overlay_conflict_tests;

#[cfg(test)]
#[path = "session_admission/comparison_scope_tests.rs"]
mod comparison_scope_tests;

#[cfg(test)]
mod principal_comparison_tests;
