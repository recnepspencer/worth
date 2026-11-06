//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct TemporalModel;
impl Fragment for TemporalModel {
    const NAME: &'static str = "temporal_model";
    type Bindings = TemporalModelBindings;
    type Members = (ExternalMapping, Principal, TemporalIntent, UnrelatedRecord);
}
pub struct TemporalModelBindings;
impl BindingSet for TemporalModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<declaration::authentication::WorthQueryExternalPrincipalIdentity>
    for TemporalModelBindings
{
    type Binding = declaration::authentication::WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<declaration::authentication::WorthQueryPrincipalMappingStatus>
    for TemporalModelBindings
{
    type Binding = declaration::authentication::WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for TemporalModelBindings {
    type Binding = declaration::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<String> for TemporalModelBindings {
    type Binding = declaration::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<TemporalHostSchema, Self> {
        entity(TemporalModel, Self)
    }
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<TemporalHostSchema, Self> {
        entity(TemporalModel, Self)
    }
}
pub struct TemporalIntent;
impl Entity for TemporalIntent {
    const NAME: &'static str = "TemporalIntent";
    type Aspects = (IntentFacts,);
    type Relations = (IntentLiveTarget,);
}
impl TemporalIntent {
    pub fn reference() -> ApplicationEntityRef<TemporalHostSchema, Self> {
        entity(TemporalModel, Self)
    }
}
pub struct UnrelatedRecord;
impl Entity for UnrelatedRecord {
    const NAME: &'static str = "UnrelatedRecord";
    type Aspects = (UnrelatedFacts,);
    type Relations = ();
}
impl UnrelatedRecord {
    pub fn reference() -> ApplicationEntityRef<TemporalHostSchema, Self> {
        entity(TemporalModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<TemporalModel, ExternalMapping, Self> {
        aspect(TemporalModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: declaration::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: declaration::authentication::WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for TemporalModelBindings {
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
    pub fn reference() -> AspectSelection<TemporalModel, Principal, Self> {
        aspect(TemporalModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalFactsValue> for TemporalModelBindings {
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
pub struct IntentFacts;
impl Aspect for IntentFacts {
    const NAME: &'static str = "IntentFacts";
    type Row = IntentFactsValue;
}
impl IntentFacts {
    pub fn reference() -> AspectSelection<TemporalModel, TemporalIntent, Self> {
        aspect(TemporalModel, TemporalIntent, Self)
    }
}
pub struct IntentFactsValue {
    pub intent_identity_field: String,
    pub intent_revision_field: u64,
    pub intent_due_field: u64,
    pub intent_lifecycle_field: String,
    pub intent_input_field: String,
    pub intent_gate_field: String,
    pub intent_effect_field: String,
}
impl Record<IntentFactsValue> for TemporalModelBindings {
    fn record() -> impl Shape<IntentFactsValue, Self> {
        field("IntentIdentityField", |row: &IntentFactsValue| {
            row.intent_identity_field.clone()
        })
        .read_only()
        .and("IntentRevisionField", |row: &IntentFactsValue| {
            row.intent_revision_field
        })
        .and("IntentDueField", |row: &IntentFactsValue| {
            row.intent_due_field
        })
        .and("IntentLifecycleField", |row: &IntentFactsValue| {
            row.intent_lifecycle_field.clone()
        })
        .and("IntentInputField", |row: &IntentFactsValue| {
            row.intent_input_field.clone()
        })
        .and("IntentGateField", |row: &IntentFactsValue| {
            row.intent_gate_field.clone()
        })
        .and("IntentEffectField", |row: &IntentFactsValue| {
            row.intent_effect_field.clone()
        })
        .build(
            |intent_identity_field,
             intent_revision_field,
             intent_due_field,
             intent_lifecycle_field,
             intent_input_field,
             intent_gate_field,
             intent_effect_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IntentFactsValue {
                    intent_identity_field,
                    intent_revision_field,
                    intent_due_field,
                    intent_lifecycle_field,
                    intent_input_field,
                    intent_gate_field,
                    intent_effect_field,
                })
            },
        )
    }
}
pub struct UnrelatedFacts;
impl Aspect for UnrelatedFacts {
    const NAME: &'static str = "UnrelatedFacts";
    type Row = UnrelatedFactsValue;
}
impl UnrelatedFacts {
    pub fn reference() -> AspectSelection<TemporalModel, UnrelatedRecord, Self> {
        aspect(TemporalModel, UnrelatedRecord, Self)
    }
}
pub struct UnrelatedFactsValue {
    pub unrelated_value_field: u64,
}
impl Record<UnrelatedFactsValue> for TemporalModelBindings {
    fn record() -> impl Shape<UnrelatedFactsValue, Self> {
        field("UnrelatedValueField", |row: &UnrelatedFactsValue| {
            row.unrelated_value_field
        })
        .read_only()
        .build(|unrelated_value_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(UnrelatedFactsValue {
                unrelated_value_field,
            })
        })
    }
}
pub type ExternalIdentityField = FieldSelection<
    TemporalModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    TemporalModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<TemporalModel, Principal, PrincipalFacts, u64>;
pub type IntentIdentityField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, String>;
pub type IntentRevisionField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, u64>;
pub type IntentDueField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, u64>;
pub type IntentLifecycleField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, String>;
pub type IntentInputField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, String>;
pub type IntentGateField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, String>;
pub type IntentEffectField = FieldSelection<TemporalModel, TemporalIntent, IntentFacts, String>;
pub type UnrelatedValueField = FieldSelection<TemporalModel, UnrelatedRecord, UnrelatedFacts, u64>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<TemporalHostSchema, Self, ExternalMapping, Principal> {
        relation(TemporalModel, ExternalMapping, Self)
    }
}
pub struct IntentLiveTarget;
impl Relation for IntentLiveTarget {
    const NAME: &'static str = "IntentLiveTarget";
    type To = TemporalIntent;
}
impl IntentLiveTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<TemporalHostSchema, Self, TemporalIntent, TemporalIntent> {
        relation(TemporalModel, TemporalIntent, Self)
    }
}
