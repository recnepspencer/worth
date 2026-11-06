pub use std::sync::Arc;

pub use worth_query_host::facade::{declaration, domain, primary_graph};

pub use super::super::adapters::{FinancialInvoker, FinancialPrincipalSource};
pub use super::super::schema::*;

pub fn execution(
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
    MarketIdentityField,
    String,
    MarketRevisionField,
    u64,
    MarketLifecycleField,
    String,
    primary_graph::WorthQueryPublicTemporalOperationAuthorization,
> {
    primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
        invariant,
        FinancialInvoker,
        MarketIdentityField::at("MarketIdentityField"),
        MarketRevisionField::at("MarketRevisionField"),
        MarketLifecycleField::at("MarketLifecycleField"),
        "active".to_string(),
        "completed".to_string(),
        primary_graph::WorthQueryPublicTemporalOperationAuthorization,
    )
    .unwrap()
}

pub fn reconstruction(
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
    MarketIdentityField,
    String,
    FinancialPrincipalSource,
    primary_graph::WorthQueryPublicTemporalQueryAuthorization,
> {
    primary_graph::WorthQueryTemporalReconstructionAccess::new(
        principal_binding,
        FinancialPrincipalSource(authentication),
        MarketIdentityField::at("MarketIdentityField"),
        record_identity.to_string(),
    )
    .unwrap()
}
