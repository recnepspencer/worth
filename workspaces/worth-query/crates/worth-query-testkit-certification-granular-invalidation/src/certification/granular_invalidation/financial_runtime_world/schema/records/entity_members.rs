use super::*;

#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<FinancialHostSchema, Self> {
        entity(FinancialModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<FinancialHostSchema, Self> {
        entity(FinancialModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct MarketObservation;
impl Entity for MarketObservation {
    const NAME: &'static str = "MarketObservation";
    type Aspects = (
        MarketIdentity,
        CurveFacts,
        VolatilityFacts,
        PriceFacts,
        RiskFacts,
        PortfolioFacts,
        AuditFacts,
    );
    type Relations = ();
}
impl MarketObservation {
    pub fn reference() -> ApplicationEntityRef<FinancialHostSchema, Self> {
        entity(FinancialModel, Self)
    }
}
