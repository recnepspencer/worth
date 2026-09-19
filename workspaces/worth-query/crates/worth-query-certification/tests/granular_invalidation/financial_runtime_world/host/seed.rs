use worth_query_host::facade::{declaration, domain, primary_graph};

use super::super::schema::*;

pub(super) fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<FinancialHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        FinancialHostSchema,
        FinancialPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
    >,
    record_identity: &str,
    curve_rate: u64,
    quote_mid: u64,
    risk_value: u64,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("financial-courtroom").unwrap(),
            1_u64,
            declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "financial-courtroom",
            )
            .unwrap(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                MarketObservation::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("market-row-1").unwrap(),
            )
            .field(
                MarketIdentityField::reference(),
                record_identity.to_string(),
            )
            .field(MarketRevisionField::reference(), 1_u64)
            .field(MarketDueField::reference(), 10_u64)
            .field(MarketLifecycleField::reference(), "active".to_string())
            .field(MarketInputField::reference(), risk_value.to_string())
            .field(MarketGateField::reference(), "ready".to_string())
            .field(CurvePartitionField::reference(), "usd-rates".to_string())
            .field(CurveDetailField::reference(), "5y".to_string())
            .field(CurveZeroRateField::reference(), curve_rate)
            .field(VolatilitySurfaceField::reference(), 240_u64)
            .field(QuoteMidField::reference(), quote_mid)
            .field(RiskValueField::reference(), risk_value)
            .field(PortfolioValueField::reference(), risk_value)
            .field(PortfolioDeskField::reference(), "rates".to_string())
            .field(PortfolioRankField::reference(), 1_u64)
            .field(AuditLabelField::reference(), "market-audit".to_string()),
        )
        .unwrap();
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                MarketObservation::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("market-row-10y").unwrap(),
            )
            .field(
                MarketIdentityField::reference(),
                "curve-usd-rates-10y".to_string(),
            )
            .field(MarketRevisionField::reference(), 1_u64)
            .field(MarketDueField::reference(), 10_u64)
            .field(MarketLifecycleField::reference(), "active".to_string())
            .field(MarketInputField::reference(), risk_value.to_string())
            .field(MarketGateField::reference(), "ready".to_string())
            .field(CurvePartitionField::reference(), "usd-rates".to_string())
            .field(CurveDetailField::reference(), "10y".to_string())
            .field(CurveZeroRateField::reference(), curve_rate + 25)
            .field(VolatilitySurfaceField::reference(), 260_u64)
            .field(QuoteMidField::reference(), quote_mid)
            .field(RiskValueField::reference(), risk_value)
            .field(PortfolioValueField::reference(), risk_value)
            .field(PortfolioDeskField::reference(), "rates".to_string())
            .field(PortfolioRankField::reference(), 18_u64)
            .field(AuditLabelField::reference(), "market-audit-10y".to_string()),
        )
        .unwrap();
}
