//! Conditional projections reuse the completed record declarations.
use super::super::schema::*;
use worth_query_host::facade::declaration::application_schema::{
    fragment::RecordSelection, ApplicationSchema,
};
pub use worth_query_host::facade::domain::{
    self, AspectContract, AspectMask, FieldKey, ProjectionMask,
};
fn contract<F: RecordSelection>(field: F) -> AspectContract {
    let declaration = <FinancialHostSchema as ApplicationSchema>::declaration().unwrap();
    domain::declared_application_aspect(declaration.erased(), field.entity(), field.aspect())
        .unwrap()
        .contract()
        .clone()
}
pub fn curve_contract() -> AspectContract {
    contract(CurveZeroRateField::at("CurveZeroRateField"))
}
pub fn volatility_contract() -> AspectContract {
    contract(VolatilitySurfaceField::at("VolatilitySurfaceField"))
}
pub fn audit_contract() -> AspectContract {
    contract(AuditLabelField::at("AuditLabelField"))
}
pub fn price_contract() -> AspectContract {
    contract(QuoteMidField::at("QuoteMidField"))
}
pub fn risk_contract() -> AspectContract {
    contract(RiskValueField::at("RiskValueField"))
}
pub fn portfolio_contract() -> AspectContract {
    contract(PortfolioValueField::at("PortfolioValueField"))
}
pub fn field_mask(field: &'static str) -> AspectMask<ProjectionMask> {
    AspectMask::new([domain::CanonicalFieldPath::single(
        FieldKey::new(field).unwrap(),
    )])
}
pub fn projection_contract(
    contract: AspectContract,
    field: &'static str,
) -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(contract, field_mask(field)).unwrap()
}
