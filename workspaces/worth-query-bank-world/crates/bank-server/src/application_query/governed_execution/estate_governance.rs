use bank_domain::model::BankPrincipalId;
use bank_domain::queries::{EstateGovernanceQuery, EstateGovernanceRequest};
use bank_domain::reads::EstateGovernanceContext;
use bank_domain::schema::{
    BankSchema, EstateCase, EstateCaseIdentityField, Principal, ViewEstateAdministrationCapability,
    ViewRestrictedEstateOperation,
};
use worth_query_host::facade::declaration::application_query::ApplicationQueryParameterSet;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOneShotResult, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode,
};

use super::super::BankApplicationQueryDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankReadControls};

pub(crate) fn execute_estate_governance(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: EstateGovernanceRequest,
    controls: &BankReadControls,
) -> Result<
    WorthQueryApplicationOneShotResult<EstateGovernanceQuery, EstateGovernanceContext>,
    BankApplicationQueryDenial,
> {
    let application = runtime.application_runtime();
    let selected = application
        .on_branch(application.current_world())
        .select()
        .map_err(BankApplicationQueryDenial::from_product_selection)?;
    let query = application
        .installed_schema()
        .application_query(EstateGovernanceQuery::reference())
        .map_err(BankApplicationQueryDenial::from_installation)?;
    let capability = application
        .installed_schema()
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .map_err(BankApplicationQueryDenial::from_capability_installation)?;
    let capability_access = selected
        .admit_capability_access(
            principal.query(),
            &capability,
            request.capability_request(),
            controls.request(),
        )
        .map_err(BankApplicationQueryDenial::from_capability_admission)?;
    let scope = selected
        .resolve_entity(
            EstateCaseIdentityField::reference(),
            request.estate(),
            controls.request(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
    let access = WorthQueryApplicationQueryAccessContext::<
        BankSchema,
        Principal,
        BankPrincipalId,
        EstateCase,
    >::new(principal.query(), &scope);
    let plan = selected
        .admit_governed_application_query(
            &query,
            &access,
            capability_access,
            ApplicationQueryParameterSet::<EstateGovernanceQuery>::new(),
            controls.application_query_controls(),
        )
        .map_err(BankApplicationQueryDenial::from_admission)?;

    application
        .execute_application_query_one_shot(plan)
        .map_err(BankApplicationQueryDenial::from_execution)
}
