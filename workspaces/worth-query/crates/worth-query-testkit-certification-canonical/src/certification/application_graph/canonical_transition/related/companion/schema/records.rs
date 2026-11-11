//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{fragment::*, *};
#[derive(Clone, Copy)]
pub struct CompanionModel;
impl Fragment for CompanionModel {
    const NAME: &'static str = "companion";
    type Bindings = CompanionModelBindings;
    type Members = (Companion,);
}
pub struct CompanionModelBindings;
impl BindingSet for CompanionModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for CompanionModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct Companion;
impl Entity for Companion {
    const NAME: &'static str = "Companion";
    type Aspects = (CompanionFacts,);
    type Relations = (CompanionLink,);
}
impl Companion {
    pub fn reference() -> ApplicationEntityRef<CanonicalSchema, Self> {
        entity(CompanionModel, Self)
    }
}
pub struct CompanionFacts;
impl Aspect for CompanionFacts {
    const NAME: &'static str = "CompanionFacts";
    type Row = CompanionFactsValue;
}
impl CompanionFacts {
    pub fn reference() -> AspectSelection<CompanionModel, Companion, Self> {
        aspect(CompanionModel, Companion, Self)
    }
}
pub struct CompanionFactsValue {
    pub companion_identity_field: String,
    pub companion_value_field: String,
}
impl Record<CompanionFactsValue> for CompanionModelBindings {
    fn record() -> impl Shape<CompanionFactsValue, Self> {
        field("CompanionIdentityField", |row: &CompanionFactsValue| {
            row.companion_identity_field.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("CompanionValueField", |row: &CompanionFactsValue| {
            row.companion_value_field.clone()
        })
        .equality_queryable(true)
        .build(
            |companion_identity_field, companion_value_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CompanionFactsValue {
                    companion_identity_field,
                    companion_value_field,
                })
            },
        )
    }
}
pub type CompanionIdentityField = FieldSelection<CompanionModel, Companion, CompanionFacts, String>;
pub type CompanionValueField = FieldSelection<CompanionModel, Companion, CompanionFacts, String>;
pub struct CompanionLink;
impl Relation for CompanionLink {
    const NAME: &'static str = "CompanionLink";
    type To = Companion;
}
impl CompanionLink {
    pub fn reference() -> ApplicationRelationRef<CanonicalSchema, Self, Companion, Companion> {
        relation(CompanionModel, Companion, Self)
    }
}
