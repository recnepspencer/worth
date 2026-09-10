use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::WorthQueryApplicationLiveLease;

impl<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
    WorthQueryApplicationLiveLease<
        '_,
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    pub(super) fn terminate(
        &mut self,
        disposition: BridgeExecutionBasisTerminalDisposition,
    ) -> bool {
        let Some(mut basis) = self.basis.take() else {
            return true;
        };
        if self.queue.release_all(&mut basis).is_err() {
            self.basis = Some(basis);
            return false;
        }
        let crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis {
            bridge,
            relational,
            product_observation,
        } = basis;
        match bridge.finalize(disposition) {
            Ok(_) => {
                let Some(graph_work) = self.graph_work.take() else {
                    return self.read_completion.is_some();
                };
                let Some(read_proof) = self.read_proof.take() else {
                    return false;
                };
                let Some(observed) = self.initial_read_work.take() else {
                    return false;
                };
                let Some(basis_release) = self.basis_release.take() else {
                    return false;
                };
                match graph_work.complete_query_read(read_proof, observed, basis_release) {
                    Ok(completion) => {
                        self.read_completion = Some(completion);
                        true
                    }
                    Err(_) => false,
                }
            }
            Err(failure) => {
                self.basis = Some(
                    crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis {
                        bridge: failure.into_basis(),
                        relational,
                        product_observation,
                    },
                );
                false
            }
        }
    }
}

impl<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    > Drop
    for WorthQueryApplicationLiveLease<
        '_,
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    fn drop(&mut self) {
        let _ = self.terminate(BridgeExecutionBasisTerminalDisposition::Abandoned);
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
