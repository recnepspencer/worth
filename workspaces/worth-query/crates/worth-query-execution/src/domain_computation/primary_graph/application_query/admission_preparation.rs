use worth_query_admission::facade::application_query::admit_application_query_parameters;
use worth_query_admission::facade::application_query::readmit_application_query_parameters;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::control_validation::validate_controls;
use super::WorthQueryApplicationQueryAccessContext;
use super::WorthQueryApplicationQueryAdmissionDenial;
use super::WorthQueryApplicationQueryAdmissionDenialKind;
use super::WorthQueryApplicationQueryControls;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod fresh_source;

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
        self.validate_application_query_admission(query, access, &controls)?;
        let parameters =
            admit_application_query_parameters(query, parameters).map_err(|denial| {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::Parameter(denial.kind()),
                    denial.parameter(),
                )
            })?;
        Ok((parameters, controls))
    }

    pub(super) fn prepare_application_query_source_readmission<
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
        retained: &WorthQueryAdmittedApplicationQueryParameters,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        (
            WorthQueryAdmittedApplicationQueryParameters,
            WorthQueryApplicationQueryControls<'a, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        self.validate_application_query_admission(query, access, &controls)?;

        let parameters =
            readmit_application_query_parameters(query, retained).map_err(|denial| {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::Parameter(denial.kind()),
                    denial.parameter(),
                )
            })?;
        Ok((parameters, controls))
    }

    fn validate_application_query_admission<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
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
        controls: &WorthQueryApplicationQueryControls<'_, Schema>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        validate_admission_request(controls.request_scope(), query.name())?;

        self.validate_installed_query(query)?;
        self.validate_access_authority(query, access)?;
        validate_controls(query, controls)
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
mod tests;
