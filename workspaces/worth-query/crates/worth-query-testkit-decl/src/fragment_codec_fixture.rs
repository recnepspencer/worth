//! One checked scalar representation used by owned and extended record fields.
use super::fragment_fixture::{row_sample, scalar_sample, BindingSample, Bindings};
use std::any::TypeId;
use worth_query_decl::facade::application_schema::{fragment::*, *};

pub struct OwnedValue;
pub struct ExtendedValue;
impl Aspect for OwnedValue {
    const NAME: &'static str = "owned_value";
    type Row = StoredValue;
}
impl Aspect for ExtendedValue {
    const NAME: &'static str = "extended_value";
    type Row = StoredValue;
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Number {
    pub number: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredValue {
    pub value: Number,
}
worth_query_decl::facade::worth_query_value_binding!(pub NumberBinding for Number {
    identity: "fixture.stored.number.v1", scalar: String, canonical,
});
impl Scalar<Number> for Bindings {
    type Binding = NumberBinding;
}
impl Record<StoredValue> for Bindings {
    fn record() -> impl Shape<StoredValue, Self> {
        field("value", |value: &StoredValue| value.value.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(StoredValue { value })
            },
        )
    }
}
pub(crate) fn samples(value: TypeId) -> Option<Vec<Box<dyn BindingSample>>> {
    if value == TypeId::of::<Number>() {
        return Some(vec![scalar_sample(Number { number: 7 })]);
    }
    if value == TypeId::of::<StoredValue>() {
        return Some(vec![row_sample(StoredValue {
            value: Number { number: 7 },
        })]);
    }
    None
}
