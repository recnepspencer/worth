use super::*;

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<FinancialModel, ExternalMapping, Self> {
        aspect(FinancialModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: declaration::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: declaration::authentication::WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for FinancialModelBindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<FinancialModel, Principal, Self> {
        aspect(FinancialModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct MarketIdentity;
impl Aspect for MarketIdentity {
    const NAME: &'static str = "MarketIdentity";
    type Row = MarketIdentityValue;
}
impl MarketIdentity {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct MarketIdentityValue {
    pub market_identity_field: String,
    pub market_revision_field: u64,
    pub market_due_field: u64,
    pub market_lifecycle_field: String,
    pub market_input_field: String,
    pub market_gate_field: String,
}
impl Record<MarketIdentityValue> for FinancialModelBindings {
    fn record() -> impl Shape<MarketIdentityValue, Self> {
        field("MarketIdentityField", |row: &MarketIdentityValue| {
            row.market_identity_field.clone()
        })
        .read_only()
        .and("MarketRevisionField", |row: &MarketIdentityValue| {
            row.market_revision_field
        })
        .and("MarketDueField", |row: &MarketIdentityValue| {
            row.market_due_field
        })
        .and("MarketLifecycleField", |row: &MarketIdentityValue| {
            row.market_lifecycle_field.clone()
        })
        .and("MarketInputField", |row: &MarketIdentityValue| {
            row.market_input_field.clone()
        })
        .and("MarketGateField", |row: &MarketIdentityValue| {
            row.market_gate_field.clone()
        })
        .build(
            |market_identity_field,
             market_revision_field,
             market_due_field,
             market_lifecycle_field,
             market_input_field,
             market_gate_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MarketIdentityValue {
                    market_identity_field,
                    market_revision_field,
                    market_due_field,
                    market_lifecycle_field,
                    market_input_field,
                    market_gate_field,
                })
            },
        )
    }
}
pub struct CurveFacts;
impl Aspect for CurveFacts {
    const NAME: &'static str = "CurveFacts";
    type Row = CurveFactsValue;
}
impl CurveFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct CurveFactsValue {
    pub curve_partition_field: String,
    pub curve_detail_field: String,
    pub curve_zero_rate_field: u64,
}
impl Record<CurveFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<CurveFactsValue, Self> {
        field("CurvePartitionField", |row: &CurveFactsValue| {
            row.curve_partition_field.clone()
        })
        .read_only()
        .and("CurveDetailField", |row: &CurveFactsValue| {
            row.curve_detail_field.clone()
        })
        .read_only()
        .and("CurveZeroRateField", |row: &CurveFactsValue| {
            row.curve_zero_rate_field
        })
        .build(
            |curve_partition_field,
             curve_detail_field,
             curve_zero_rate_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CurveFactsValue {
                    curve_partition_field,
                    curve_detail_field,
                    curve_zero_rate_field,
                })
            },
        )
    }
}
pub struct VolatilityFacts;
impl Aspect for VolatilityFacts {
    const NAME: &'static str = "VolatilityFacts";
    type Row = VolatilityFactsValue;
}
impl VolatilityFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct VolatilityFactsValue {
    pub volatility_surface_field: u64,
}
impl Record<VolatilityFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<VolatilityFactsValue, Self> {
        field("VolatilitySurfaceField", |row: &VolatilityFactsValue| {
            row.volatility_surface_field
        })
        .read_only()
        .build(
            |volatility_surface_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(VolatilityFactsValue {
                    volatility_surface_field,
                })
            },
        )
    }
}
pub struct PriceFacts;
impl Aspect for PriceFacts {
    const NAME: &'static str = "PriceFacts";
    type Row = PriceFactsValue;
}
impl PriceFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct PriceFactsValue {
    pub quote_mid_field: u64,
}
impl Record<PriceFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<PriceFactsValue, Self> {
        field("QuoteMidField", |row: &PriceFactsValue| row.quote_mid_field).build(
            |quote_mid_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PriceFactsValue { quote_mid_field })
            },
        )
    }
}
pub struct RiskFacts;
impl Aspect for RiskFacts {
    const NAME: &'static str = "RiskFacts";
    type Row = RiskFactsValue;
}
impl RiskFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct RiskFactsValue {
    pub risk_value_field: u64,
}
impl Record<RiskFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<RiskFactsValue, Self> {
        field("RiskValueField", |row: &RiskFactsValue| {
            row.risk_value_field
        })
        .build(|risk_value_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(RiskFactsValue { risk_value_field })
        })
    }
}
pub struct PortfolioFacts;
impl Aspect for PortfolioFacts {
    const NAME: &'static str = "PortfolioFacts";
    type Row = PortfolioFactsValue;
}
impl PortfolioFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct PortfolioFactsValue {
    pub portfolio_value_field: u64,
    pub portfolio_desk_field: String,
    pub portfolio_rank_field: u64,
}
impl Record<PortfolioFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<PortfolioFactsValue, Self> {
        field("PortfolioValueField", |row: &PortfolioFactsValue| {
            row.portfolio_value_field
        })
        .and("PortfolioDeskField", |row: &PortfolioFactsValue| {
            row.portfolio_desk_field.clone()
        })
        .and("PortfolioRankField", |row: &PortfolioFactsValue| {
            row.portfolio_rank_field
        })
        .build(
            |portfolio_value_field,
             portfolio_desk_field,
             portfolio_rank_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PortfolioFactsValue {
                    portfolio_value_field,
                    portfolio_desk_field,
                    portfolio_rank_field,
                })
            },
        )
    }
}
pub struct AuditFacts;
impl Aspect for AuditFacts {
    const NAME: &'static str = "AuditFacts";
    type Row = AuditFactsValue;
}
impl AuditFacts {
    pub fn reference() -> AspectSelection<FinancialModel, MarketObservation, Self> {
        aspect(FinancialModel, MarketObservation, Self)
    }
}
pub struct AuditFactsValue {
    pub audit_label_field: String,
}
impl Record<AuditFactsValue> for FinancialModelBindings {
    fn record() -> impl Shape<AuditFactsValue, Self> {
        field("AuditLabelField", |row: &AuditFactsValue| {
            row.audit_label_field.clone()
        })
        .read_only()
        .build(|audit_label_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AuditFactsValue { audit_label_field })
        })
    }
}
