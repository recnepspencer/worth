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
            .security_observation_for(self.product())
            .map_err(|_| foreign_basis(query_name))?;
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
                controls.maximum_work,
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
        let (application, controls) =
            self.retained_query_controls(retained, query.name(), controls)?;
        application
            .admit_governed_application_query(query, access, capability, parameters, controls)
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
    fn retained_query_rejects_security_from_a_sibling_branch() {
        let world = installed_authorization_world(true);
        let application = &world.application;
        let root = application.current_world();
        let sibling = application
            .branches()
            .fork(root)
            .components(|components| components.reuse_exact_relational_basis().fork_signal())
            .create()
            .expect("a distinct sibling branch should exist");
        let security = application.on_branch(sibling).select().unwrap();
        let retained = application.on_branch(root).select().unwrap();
        let scope = live_scope();
        let controls = WorthQueryProductQueryControls::new(
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(64).unwrap(),
            &scope,
        );
        let denied = security.retained_query_controls(retained, "sibling", controls);
        assert!(matches!(
            denied,
            Err(error)
                if error.kind() == WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
        ));
    }
}
