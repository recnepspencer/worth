use super::*;

pub type ExternalIdentityField = FieldSelection<
    FinancialModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    FinancialModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<FinancialModel, Principal, PrincipalFacts, u64>;
pub type MarketIdentityField =
    FieldSelection<FinancialModel, MarketObservation, MarketIdentity, String>;
pub type MarketRevisionField =
    FieldSelection<FinancialModel, MarketObservation, MarketIdentity, u64>;
pub type MarketDueField = FieldSelection<FinancialModel, MarketObservation, MarketIdentity, u64>;
pub type MarketLifecycleField =
    FieldSelection<FinancialModel, MarketObservation, MarketIdentity, String>;
pub type MarketInputField =
    FieldSelection<FinancialModel, MarketObservation, MarketIdentity, String>;
pub type MarketGateField =
    FieldSelection<FinancialModel, MarketObservation, MarketIdentity, String>;
pub type CurvePartitionField =
    FieldSelection<FinancialModel, MarketObservation, CurveFacts, String>;
pub type CurveDetailField = FieldSelection<FinancialModel, MarketObservation, CurveFacts, String>;
pub type CurveZeroRateField = FieldSelection<FinancialModel, MarketObservation, CurveFacts, u64>;
pub type VolatilitySurfaceField =
    FieldSelection<FinancialModel, MarketObservation, VolatilityFacts, u64>;
pub type QuoteMidField = FieldSelection<FinancialModel, MarketObservation, PriceFacts, u64>;
pub type RiskValueField = FieldSelection<FinancialModel, MarketObservation, RiskFacts, u64>;
pub type PortfolioValueField =
    FieldSelection<FinancialModel, MarketObservation, PortfolioFacts, u64>;
pub type PortfolioDeskField =
    FieldSelection<FinancialModel, MarketObservation, PortfolioFacts, String>;
pub type PortfolioRankField =
    FieldSelection<FinancialModel, MarketObservation, PortfolioFacts, u64>;
pub type AuditLabelField = FieldSelection<FinancialModel, MarketObservation, AuditFacts, String>;
