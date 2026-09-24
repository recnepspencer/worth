use worth_query_admission::facade::{
    application_query::{
        admit_application_query_parameters, WorthQueryAdmittedApplicationQueryParameters,
    },
    authenticated_principal::{WorthQueryRequestInterruption, WorthQueryRequestScope},
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::{
    control_validation::validate_controls, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn prepare_application_query_admission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        (
            WorthQueryAdmittedApplicationQueryParameters,
            WorthQueryApplicationQueryControls<'a, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        validate_admission_request(controls.request_scope(), query.name())?;
        self.validate_installed_query(query)?;
        self.validate_access_authority(query, access)?;
        validate_controls(query, &controls)?;
        let parameters =
            admit_application_query_parameters(query, parameters).map_err(|denial| {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::Parameter(denial.kind()),
                    denial.parameter(),
                )
            })?;
        Ok((parameters, controls))
    }

    fn validate_installed_query<Query, Parameters, QueryResult, Scope>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        if !self.installed_schema_is_current() {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                    worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
                ),
                query.name(),
            ));
        }
        self.installed_schema
            .validate_installed_query(query)
            .map_err(|denial| {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(denial.kind()),
                    denial.subject(),
                )
            })
    }

    fn validate_access_authority<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        if self.authentication_is_expired(access.principal().valid_until()) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                access.principal().binding(),
            ));
        }
        if access.principal().runtime_authority() != self.runtime.authority_identity() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignPrincipal,
                query.name(),
            ));
        }
        if access.principal().binding_identity() != &self.installed_schema.binding_identity() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                query.name(),
            ));
        }
        let scope = access.scope();
        let authority = self.runtime.authority_identity();
        if scope.runtime_authority() != authority {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignScope,
                query.name(),
            ));
        }
        if scope.binding_identity() != query.binding_identity()
            || scope.binding_identity() != &self.installed_schema.binding_identity()
        {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            ));
        }
        if scope.entity_name() != query.scope_entity() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ScopeTypeMismatch,
                scope.entity_name(),
            ));
        }
        self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        Ok(())
    }
}

pub(super) fn validate_admission_request(
    request: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
            subject,
        )),
        None => Ok(()),
    }
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

    use super::WorthQueryApplicationQueryAdmissionDenialKind;
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_capability_live_world_with_label, GovernedLiveAccountActivityQuery,
    };

    #[test]
    fn published_schema_validation_keeps_query_and_generation_authority() {
        let mut world = installed_capability_live_world_with_label("installed");
        let local_query = world
            .application
            .installed_schema()
            .certification_query(GovernedLiveAccountActivityQuery::reference())
            .unwrap();
        assert!(world
            .application
            .validate_installed_query(&local_query)
            .is_ok());

        let foreign = installed_capability_live_world_with_label("foreign");
        let foreign_query = foreign
            .application
            .installed_schema()
            .certification_query(GovernedLiveAccountActivityQuery::reference())
            .unwrap();
        assert_eq!(
            world
                .application
                .validate_installed_query(&foreign_query)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                WorthQueryApplicationQueryInstallationDenialKind::ForeignRuntime,
            )
        );

        let rebuilt = Arc::new(world.application.runtime.installed_packages().rebuild());
        world
            .application
            .runtime
            .replace_rebuilt_installation(rebuilt)
            .unwrap();
        assert!(world
            .application
            .validate_installed_query(&local_query)
            .is_ok());

        let successor = Arc::new(
            world
                .application
                .runtime
                .installed_packages()
                .successor_generation(),
        );
        world
            .application
            .runtime
            .commit_successor_installation(successor)
            .unwrap();
        assert_eq!(
            world
                .application
                .validate_installed_query(&local_query)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
            )
        );
    }
}
