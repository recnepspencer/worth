use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

use super::{WorthQueryProductQueryControls, WorthQuerySelectedProductOperation};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(super) fn retained_query_controls(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query_name: &str,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationQueryControls<'runtime, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        if self.product().branch_identity() != retained.product().branch_identity() {
            return Err(foreign_basis(query_name));
        }
        let security = self
            .application()
            .product_runtime
            .security_observation_for(execution, self.product())
            .map_err(|stop| super::selection_denial(stop, query_name))?;
        let (application, _current_security, _) = self.into_parts();
        let (retained_application, product, application_basis) = retained.into_parts();
        if !std::ptr::eq(application, retained_application) {
            return Err(foreign_basis(query_name));
        }
        Ok((
            application,
            WorthQueryApplicationQueryControls::retained_product_one_shot(
                security,
                product,
                application_basis,
                controls.maximum_results,
                controls.request,
            ),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit_retained_governed_application_query<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query: &'runtime WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    >
    where
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, controls) =
            self.retained_query_controls(execution, retained, query.name(), controls)?;
        application.admit_governed_application_query(
            execution, query, access, capability, parameters, controls,
        )
    }
}

fn foreign_basis(query_name: &str) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
        query_name,
    )
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope,
    };

    #[test]
    fn retained_and_batch_selection_keep_original_interruption_causes() {
        use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
        for (stop, expected) in [
            (
                Stop::Cancelled,
                WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
            ),
            (
                Stop::TimedOut,
                WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
            ),
        ] {
            let denial = super::super::selection_denial(Branch::Stopped(Err(stop)), "batch-stop");
            assert_eq!(denial.kind(), expected);
        }
    }

    #[test]
    fn retained_query_rejects_security_from_a_sibling_branch() {
        let world = installed_authorization_world(true);
        let application = &world.application;
        let root = application.current_world();
        let sibling = world
            .application
            .with_host_advancement(|phase| {
                application
                    .branches()
                    .fork(root)
                    .components(|components| {
                        components.reuse_exact_relational_basis().fork_signal()
                    })
                    .create(&phase)
                    .expect("a distinct sibling branch should exist")
            })
            .expect("the fixture read owns its installed host call");
        let security = world
            .application
            .with_host_advancement(|phase| application.on_branch(sibling).select(&phase).unwrap())
            .expect("the fixture read owns its installed host call");
        let retained = world
            .application
            .with_host_advancement(|phase| application.on_branch(root).select(&phase).unwrap())
            .expect("the fixture read owns its installed host call");
        let scope = live_scope();
        let controls = WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), &scope);
        let denied = application
            .with_host_advancement(|phase| {
                let execution = phase
                    .execution_request_for(&application.product_runtime)
                    .unwrap();
                security.retained_query_controls(&execution, retained, "sibling", controls)
            })
            .expect("original fixture host call");
        assert!(matches!(
            denied,
            Err(error)
                if error.kind() == WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
        ));
    }
}
