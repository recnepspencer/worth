//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub(super) struct RemovalRecords;
impl fragment::Fragment for RemovalRecords {
    const NAME: &'static str = "removal_records";
    type Bindings = RemovalRecordsBindings;
    type Members = (Subject,);
}
pub(super) struct RemovalRecordsBindings;
impl fragment::BindingSet for RemovalRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for RemovalRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub(super) struct Subject;
impl fragment::Entity for Subject {
    const NAME: &'static str = "Subject";
    type Aspects = (Option<OptionalFacts>,);
    type Relations = ();
}
impl Subject {
    pub fn reference() -> ApplicationEntityRef<RemovalSchema, Self> {
        entity(RemovalRecords, Self)
    }
}
pub(super) struct OptionalFacts;
impl fragment::Aspect for OptionalFacts {
    const NAME: &'static str = "OptionalFacts";
    type Row = OptionalFactsValue;
}
impl OptionalFacts {
    pub fn reference() -> AspectSelection<RemovalRecords, Subject, Self> {
        aspect(RemovalRecords, Subject, Self)
    }
}
pub(super) struct OptionalFactsValue {
    pub value: u64,
    pub sibling: u64,
}
impl fragment::Record<OptionalFactsValue> for RemovalRecordsBindings {
    fn record() -> impl fragment::Shape<OptionalFactsValue, Self> {
        fragment::field("Value", |row: &OptionalFactsValue| row.value)
            .equality_queryable(true)
            .and("Sibling", |row: &OptionalFactsValue| row.sibling)
            .equality_queryable(true)
            .build(|value, sibling, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OptionalFactsValue { value, sibling })
            })
    }
}
pub(super) type Value = FieldSelection<RemovalRecords, Subject, OptionalFacts, u64>;
pub(super) type Sibling = FieldSelection<RemovalRecords, Subject, OptionalFacts, u64>;
