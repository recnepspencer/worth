use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
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
    running: &'run mut crate::domain_computation::WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    product: crate::basis::WorthQueryProductBranchLease,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
) -> Result<WorthQueryAdmittedProviderSession<'run>, WorthQueryProviderSessionAdmissionFailure> {
    let staged = match running
        .admit_provider_execution_plan(graph)
        .and_then(|plan| plan.bind_application_product(product))
        .and_then(|plan| plan.readmit())
        .and_then(|session| session.prepare())
        .map(|prepared| prepared.bind_reads_and_effects())
    {
        Ok(staged) => staged,
        Err(_) => {
            return Err(WorthQueryProviderSessionAdmissionFailure {
                outcome: progression_denied(DenialStage::ProviderPlan),
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
                outcome: progression_denied(DenialStage::ProviderPlan),
                mutation_run,
            })
        }
    }
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
    ) -> Result<
        WorthQueryRegisteredProviderSession<'run>,
        WorthQueryProviderSessionRegistrationFailure,
    > {
        match authorization.register_provider_attempt(prepared, self.staged, attempt_basis, context)
        {
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
        authority: &super::WorthQueryApplicationCommitProgressionAuthority<
            '_,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> super::WorthQueryProviderProgressionCompletion
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        super::WorthQueryProviderProgressionCompletion {
            outcome: self.registered.progress(authority),
            cleanup: super::WorthQueryApplicationMutationCleanupOwner::ProviderBound(
                self.mutation_run,
            ),
        }
    }
}

#[cfg(test)]
#[path = "session_admission/overlay_conflict_tests.rs"]
mod overlay_conflict_tests;
