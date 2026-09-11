use std::sync::Arc;

use worth_query_host::facade::{declaration, domain, primary_graph};

use super::super::adapters::{FinancialInvoker, FinancialPrincipalSource};
use super::super::schema::*;

pub(super) fn execution(
    invariant: Arc<
        primary_graph::WorthQueryApplicationInvariantProjectionAuthority<FinancialHostSchema>,
    >,
) -> primary_graph::WorthQueryTemporalOperationExecution<
    FinancialHostSchema,
    ExecuteFinancial,
    FinancialInput,
    MarketObservation,
    FinancialInvoker,
    MarketObservation,
    MarketIdentity,
    MarketIdentityField,
    String,
    declaration::application_schema::ReadOnly,
    declaration::application_schema::NoApplicationUnit,
    MarketIdentity,
    MarketRevisionField,
    u64,
    declaration::application_schema::ReadWrite,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
    MarketIdentity,
    MarketLifecycleField,
    String,
    declaration::application_schema::ReadWrite,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
    primary_graph::WorthQueryPublicTemporalOperationAuthorization,
> {
    primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
        invariant,
        FinancialInvoker,
        MarketIdentityField::reference(),
        MarketRevisionField::reference(),
        MarketLifecycleField::reference(),
        "active".to_string(),
        "completed".to_string(),
        primary_graph::WorthQueryPublicTemporalOperationAuthorization,
    )
    .unwrap()
}

pub(super) fn reconstruction(
    principal_binding: domain::WorthQueryInstalledPrincipalBinding<
        FinancialHostSchema,
        FinancialPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
    >,
    authentication: Arc<super::super::adapters::FinancialAuthentication>,
    record_identity: &str,
) -> primary_graph::WorthQueryTemporalReconstructionAccess<
    FinancialHostSchema,
    FinancialPrincipalBinding,
    ExternalMapping,
    Principal,
    u64,
    declaration::application_schema::U64ApplicationValueBinding,
    MarketObservation,
    MarketIdentity,
    MarketIdentityField,
    String,
    declaration::application_schema::ReadOnly,
    declaration::application_schema::NoApplicationUnit,
    FinancialPrincipalSource,
    primary_graph::WorthQueryPublicTemporalQueryAuthorization,
> {
    primary_graph::WorthQueryTemporalReconstructionAccess::new(
        principal_binding,
        FinancialPrincipalSource(authentication),
        MarketIdentityField::reference(),
        record_identity.to_string(),
    )
    .unwrap()
}
