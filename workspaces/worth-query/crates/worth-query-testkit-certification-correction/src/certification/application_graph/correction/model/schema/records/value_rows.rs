use super::*;

pub struct WriteFacts;
impl Aspect for WriteFacts {
    const NAME: &'static str = "WriteFacts";
    type Row = WriteFactsValue;
}
impl WriteFacts {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct WriteFactsValue {
    pub binary: BinaryCounter,
    pub canonical: CanonicalCounter,
    pub delegated: DelegatedCounter,
}
impl Record<WriteFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<WriteFactsValue, Self> {
        field("Binary", |row: &WriteFactsValue| row.binary)
            .and("Canonical", |row: &WriteFactsValue| row.canonical.clone())
            .and("Delegated", |row: &WriteFactsValue| row.delegated)
            .build(
                |binary, canonical, delegated, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(WriteFactsValue {
                        binary,
                        canonical,
                        delegated,
                    })
                },
            )
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<NumericModel, ExternalMapping, Self> {
        aspect(NumericModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for NumericModelBindings {
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
    pub fn reference() -> AspectSelection<NumericModel, Principal, Self> {
        aspect(NumericModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalFactsValue> for NumericModelBindings {
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
pub struct NumberFacts;
impl Aspect for NumberFacts {
    const NAME: &'static str = "NumberFacts";
    type Row = NumberFactsValue;
}
impl NumberFacts {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct NumberFactsValue {
    pub key: String,
    pub primary: Counter,
    pub secondary: u64,
    pub read_enabled: bool,
    pub ceiling: u64,
    pub calculated: u64,
    pub operand: Option<u64>,
}
impl Record<NumberFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<NumberFactsValue, Self> {
        field("Key", |row: &NumberFactsValue| row.key.clone())
            .read_only()
            .and("Primary", |row: &NumberFactsValue| row.primary)
            .and("Secondary", |row: &NumberFactsValue| row.secondary)
            .and("ReadEnabled", |row: &NumberFactsValue| row.read_enabled)
            .and("Ceiling", |row: &NumberFactsValue| row.ceiling)
            .and("Calculated", |row: &NumberFactsValue| row.calculated)
            .and("Operand", |row: &NumberFactsValue| row.operand)
            .build(
                |key,
                 primary,
                 secondary,
                 read_enabled,
                 ceiling,
                 calculated,
                 operand,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(NumberFactsValue {
                        key,
                        primary,
                        secondary,
                        read_enabled,
                        ceiling,
                        calculated,
                        operand,
                    })
                },
            )
    }
}
pub struct ScalarFacts;
impl Aspect for ScalarFacts {
    const NAME: &'static str = "ScalarFacts";
    type Row = ScalarFactsValue;
}
impl ScalarFacts {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct ScalarFactsValue {
    pub value: u64,
}
impl Record<ScalarFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<ScalarFactsValue, Self> {
        field("Value", |row: &ScalarFactsValue| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ScalarFactsValue { value })
            },
        )
    }
}
pub struct ScalarResult;
impl Aspect for ScalarResult {
    const NAME: &'static str = "ScalarResult";
    type Row = ScalarResultValue;
}
impl ScalarResult {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct ScalarResultValue {
    pub total: u64,
}
impl Record<ScalarResultValue> for NumericModelBindings {
    fn record() -> impl Shape<ScalarResultValue, Self> {
        field("Total", |row: &ScalarResultValue| row.total).build(
            |total, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ScalarResultValue { total })
            },
        )
    }
}
pub struct DefinitionRevisionFacts;
impl Aspect for DefinitionRevisionFacts {
    const NAME: &'static str = "DefinitionRevisionFacts";
    type Row = DefinitionRevisionFactsValue;
}
impl DefinitionRevisionFacts {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct DefinitionRevisionFactsValue {
    pub definition_revision: u64,
}
impl Record<DefinitionRevisionFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<DefinitionRevisionFactsValue, Self> {
        field(
            "DefinitionRevision",
            |row: &DefinitionRevisionFactsValue| row.definition_revision,
        )
        .build(|definition_revision, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(DefinitionRevisionFactsValue {
                definition_revision,
            })
        })
    }
}
pub struct InputFacts;
impl Aspect for InputFacts {
    const NAME: &'static str = "InputFacts";
    type Row = InputFactsValue;
}
impl InputFacts {
    pub fn reference() -> AspectSelection<NumericModel, Number, Self> {
        aspect(NumericModel, Number, Self)
    }
}
pub struct InputFactsValue {
    pub bias: u64,
    pub sibling: u64,
}
impl Record<InputFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<InputFactsValue, Self> {
        field("Bias", |row: &InputFactsValue| row.bias)
            .and("Sibling", |row: &InputFactsValue| row.sibling)
            .build(|bias, sibling, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(InputFactsValue { bias, sibling })
            })
    }
}
pub struct GrantFacts;
impl Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantFactsValue;
}
impl GrantFacts {
    pub fn reference() -> AspectSelection<NumericModel, Grant, Self> {
        aspect(NumericModel, Grant, Self)
    }
}
pub struct GrantFactsValue {
    pub disclosure: String,
    pub grant_identity: String,
    pub grant_action: String,
    pub grant_purpose: String,
    pub grant_status: String,
    pub grant_scope_key: String,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_depth: u64,
}
impl Record<GrantFactsValue> for NumericModelBindings {
    fn record() -> impl Shape<GrantFactsValue, Self> {
        field("Disclosure", |row: &GrantFactsValue| row.disclosure.clone())
            .read_only()
            .and("GrantIdentity", |row: &GrantFactsValue| {
                row.grant_identity.clone()
            })
            .read_only()
            .and("GrantAction", |row: &GrantFactsValue| {
                row.grant_action.clone()
            })
            .read_only()
            .and("GrantPurpose", |row: &GrantFactsValue| {
                row.grant_purpose.clone()
            })
            .read_only()
            .and("GrantStatus", |row: &GrantFactsValue| {
                row.grant_status.clone()
            })
            .and("GrantScopeKey", |row: &GrantFactsValue| {
                row.grant_scope_key.clone()
            })
            .read_only()
            .and("GrantNotBefore", |row: &GrantFactsValue| {
                row.grant_not_before
            })
            .read_only()
            .and("GrantNotAfter", |row: &GrantFactsValue| row.grant_not_after)
            .read_only()
            .and("GrantDepth", |row: &GrantFactsValue| row.grant_depth)
            .read_only()
            .build(
                |disclosure,
                 grant_identity,
                 grant_action,
                 grant_purpose,
                 grant_status,
                 grant_scope_key,
                 grant_not_before,
                 grant_not_after,
                 grant_depth,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GrantFactsValue {
                        disclosure,
                        grant_identity,
                        grant_action,
                        grant_purpose,
                        grant_status,
                        grant_scope_key,
                        grant_not_before,
                        grant_not_after,
                        grant_depth,
                    })
                },
            )
    }
}
