pub use worth_query_host::facade::domain::{
    self, AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, FieldDeclaration, FieldKey, FieldRequirement,
    ProjectionMask, ScalarAspectType, StructAspectShape,
};

pub fn curve_contract() -> AspectContract {
    struct_contract(
        "CurveFacts",
        0x9161_101e,
        [
            ("CurvePartitionField", ScalarAspectType::String),
            ("CurveDetailField", ScalarAspectType::String),
            ("CurveZeroRateField", ScalarAspectType::UInt64),
        ],
    )
}

pub fn volatility_contract() -> AspectContract {
    scalar_contract(
        "VolatilityFacts",
        0x9161_101f,
        "VolatilitySurfaceField",
        ScalarAspectType::UInt64,
    )
}

pub fn audit_contract() -> AspectContract {
    scalar_contract(
        "AuditFacts",
        0x9161_1023,
        "AuditLabelField",
        ScalarAspectType::String,
    )
}

pub fn price_contract() -> AspectContract {
    scalar_contract(
        "PriceFacts",
        0x9161_1020,
        "QuoteMidField",
        ScalarAspectType::UInt64,
    )
}

pub fn risk_contract() -> AspectContract {
    scalar_contract(
        "RiskFacts",
        0x9161_1021,
        "RiskValueField",
        ScalarAspectType::UInt64,
    )
}

pub fn portfolio_contract() -> AspectContract {
    struct_contract(
        "PortfolioFacts",
        0x9161_1022,
        [
            ("PortfolioValueField", ScalarAspectType::UInt64),
            ("PortfolioDeskField", ScalarAspectType::String),
            ("PortfolioRankField", ScalarAspectType::UInt64),
        ],
    )
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

pub fn scalar_contract(
    aspect: &'static str,
    identity: u64,
    field: &'static str,
    family: ScalarAspectType,
) -> AspectContract {
    struct_contract(aspect, identity, [(field, family)])
}

pub fn struct_contract<const N: usize>(
    aspect: &'static str,
    identity: u64,
    fields: [(&'static str, ScalarAspectType); N],
) -> AspectContract {
    AspectContract::struct_aspect(
        AspectKey::new(aspect).unwrap(),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new(fields.into_iter().map(|(field, family)| {
            FieldDeclaration::new(
                FieldKey::new(field).unwrap(),
                family,
                FieldRequirement::Required,
                AbsenceLaw::Required,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap()
        }))
        .unwrap(),
    )
}
