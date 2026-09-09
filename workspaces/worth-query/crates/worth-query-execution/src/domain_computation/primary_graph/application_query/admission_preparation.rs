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
        self.runtime
            .installed_packages()
            .validate_application_schema(&self.installed_schema)
            .map_err(|denial| {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                        map_schema_denial(denial.kind()),
                    ),
                    query.name(),
                )
            })?;
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

fn map_schema_denial(
    kind: worth_query_installation::facade::WorthQueryInstalledApplicationSchemaDenialKind,
) -> worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind {
    use worth_query_installation::facade::{
        WorthQueryApplicationQueryInstallationDenialKind as Query,
        WorthQueryInstalledApplicationSchemaDenialKind as Schema,
    };
    match kind {
        Schema::ForeignRuntime => Query::ForeignRuntime,
        Schema::StaleGeneration => Query::StaleGeneration,
        Schema::PackageIdentityChanged => Query::PackageIdentityChanged,
        Schema::AuthorityMismatch => Query::AuthorityMismatch,
        _ => Query::SchemaMeaningChanged,
    }
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}
