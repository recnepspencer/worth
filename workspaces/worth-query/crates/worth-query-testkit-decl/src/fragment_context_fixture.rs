//! A foreign checked value receives the entry owner's original borrowed context.

use super::fragment_fixture::{sample_with, BindingSample};
use std::any::TypeId;
use std::num::NonZeroU64;
pub use worth_query_decl::facade::application_schema::{fragment::*, *};

pub struct Contextual;
impl Fragment for Contextual {
    const NAME: &'static str = "contextual";
    type Bindings = ContextBindings;
    type Members = (Limit,);
}
pub struct Limit;
impl Entity for Limit {
    const NAME: &'static str = "limit";
    type Aspects = (Positive,);
    type Relations = ();
}
pub struct Positive;
impl Aspect for Positive {
    const NAME: &'static str = "positive";
    type Row = NonZeroU64;
}

pub struct ValidationContext<'a> {
    pub maximum: &'a u64,
    pub admission: &'a mut ValueDecodeAdmission<'a>,
}
pub struct ContextBindings;
impl BindingSet for ContextBindings {
    type Work<'a> = ValidationContext<'a>;
    fn with_admission<Output>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> Output,
    ) -> Output {
        use_admission(&mut |units, bytes| work.admission.admit(units, bytes))
    }
}
impl Scalar<u64> for ContextBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Record<NonZeroU64> for ContextBindings {
    fn record() -> impl Shape<NonZeroU64, Self> {
        field("value", |value: &NonZeroU64| value.get()).build(
            |value, work: &mut ValidationContext<'_>| {
                work.admission
                    .admit(0, 0)
                    .map_err(|_| InvalidPositive::Admission)?;
                if value > *work.maximum {
                    return Err(InvalidPositive::OutsideOwnerRange);
                }
                NonZeroU64::new(value).ok_or(InvalidPositive::Zero)
            },
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidPositive {
    Admission,
    OutsideOwnerRange,
    Zero,
}
impl std::fmt::Display for InvalidPositive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "positive value refused: {self:?}")
    }
}
impl std::error::Error for InvalidPositive {}

pub fn samples(value: TypeId) -> Vec<Box<dyn BindingSample>> {
    if value == TypeId::of::<NonZeroU64>() {
        return vec![sample_with(
            NonZeroU64::new(7).unwrap(),
            |values, prefix, admission| {
                let maximum = u64::MAX;
                let mut work = ValidationContext {
                    maximum: &maximum,
                    admission,
                };
                <ContextBindings as Record<NonZeroU64>>::record()
                    .decode(values, prefix, &mut work)
                    .map_err(|cause| format!("{cause:?}"))
            },
        )];
    }
    if value == TypeId::of::<u64>() {
        return vec![sample_with(7u64, |values, prefix, admission| {
            let maximum = u64::MAX;
            let mut work = ValidationContext {
                maximum: &maximum,
                admission,
            };
            let raw = values.get(prefix).ok_or("missing scalar sample")?;
            ContextBindings::with_admission(&mut work, |admission| {
                U64ApplicationValueBinding::decode_admitted(raw, admission)
            })
            .map_err(|cause| format!("{cause:?}"))
        })];
    }
    Vec::new()
}
