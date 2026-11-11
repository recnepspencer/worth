pub use worth_query_host::facade::{declaration, domain, primary_graph};

pub use super::super::schema::*;

pub fn seed_graph(
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
                MarketIdentityField::at("MarketIdentityField"),
                record_identity.to_string(),
            )
            .field(MarketRevisionField::at("MarketRevisionField"), 1_u64)
            .field(MarketDueField::at("MarketDueField"), 10_u64)
            .field(
                MarketLifecycleField::at("MarketLifecycleField"),
                "active".to_string(),
            )
            .field(
                MarketInputField::at("MarketInputField"),
                risk_value.to_string(),
            )
            .field(MarketGateField::at("MarketGateField"), "ready".to_string())
            .field(
                CurvePartitionField::at("CurvePartitionField"),
                "usd-rates".to_string(),
            )
            .field(CurveDetailField::at("CurveDetailField"), "5y".to_string())
            .field(CurveZeroRateField::at("CurveZeroRateField"), curve_rate)
            .field(
                VolatilitySurfaceField::at("VolatilitySurfaceField"),
                240_u64,
            )
            .field(QuoteMidField::at("QuoteMidField"), quote_mid)
            .field(RiskValueField::at("RiskValueField"), risk_value)
            .field(PortfolioValueField::at("PortfolioValueField"), risk_value)
            .field(
                PortfolioDeskField::at("PortfolioDeskField"),
                "rates".to_string(),
            )
            .field(PortfolioRankField::at("PortfolioRankField"), 1_u64)
            .field(
                AuditLabelField::at("AuditLabelField"),
                "market-audit".to_string(),
            ),
        )
        .unwrap();
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                MarketObservation::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("market-row-10y").unwrap(),
            )
            .field(
                MarketIdentityField::at("MarketIdentityField"),
                "curve-usd-rates-10y".to_string(),
            )
            .field(MarketRevisionField::at("MarketRevisionField"), 1_u64)
            .field(MarketDueField::at("MarketDueField"), 10_u64)
            .field(
                MarketLifecycleField::at("MarketLifecycleField"),
                "active".to_string(),
            )
            .field(
                MarketInputField::at("MarketInputField"),
                risk_value.to_string(),
            )
            .field(MarketGateField::at("MarketGateField"), "ready".to_string())
            .field(
                CurvePartitionField::at("CurvePartitionField"),
                "usd-rates".to_string(),
            )
            .field(CurveDetailField::at("CurveDetailField"), "10y".to_string())
            .field(
                CurveZeroRateField::at("CurveZeroRateField"),
                curve_rate + 25,
            )
            .field(
                VolatilitySurfaceField::at("VolatilitySurfaceField"),
                260_u64,
            )
            .field(QuoteMidField::at("QuoteMidField"), quote_mid)
            .field(RiskValueField::at("RiskValueField"), risk_value)
            .field(PortfolioValueField::at("PortfolioValueField"), risk_value)
            .field(
                PortfolioDeskField::at("PortfolioDeskField"),
                "rates".to_string(),
            )
            .field(PortfolioRankField::at("PortfolioRankField"), 18_u64)
            .field(
                AuditLabelField::at("AuditLabelField"),
                "market-audit-10y".to_string(),
            ),
        )
        .unwrap();
}
