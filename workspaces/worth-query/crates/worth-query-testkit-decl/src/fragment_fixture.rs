//! Generic record declarations shared by the declaration courts.

use std::any::{Any, TypeId};
pub use worth_query_decl::facade::application_schema::{fragment::*, *};

pub struct Fixture;
impl Fragment for Fixture {
    const NAME: &'static str = "fixture";
    type Bindings = Bindings;
    type Members = (
        Item,
        SetMetric,
        super::fragment_context_fixture::Contextual,
        super::fragment_auth_fixture::Authentication,
    );
}

pub struct Item;
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (
        Metrics,
        Option<Label>,
        Exclusive<Form>,
        super::fragment_auth_fixture::ItemIdentity,
    );
    type Relations = (Next,);
}

pub struct Metrics;
impl Aspect for Metrics {
    const NAME: &'static str = "metrics";
    type Row = Measurements;
}
pub struct Label;
impl Aspect for Label {
    const NAME: &'static str = "label";
    type Row = Caption;
}
pub struct Next;
impl Relation for Next {
    const NAME: &'static str = "next";
    type To = Item;
    const SELF_EDGES: bool = false;
}

pub struct Form;
impl OneOf for Form {
    type Row = FormValue;
    type Cases = (Measured, Named);
}
pub struct Measured;
impl Aspect for Measured {
    const NAME: &'static str = "measured";
    type Row = Measurements;
}
pub struct Named;
impl Aspect for Named {
    const NAME: &'static str = "named";
    type Row = Caption;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormValue {
    Measured(Measurements),
    Named(Caption),
}
impl From<Measurements> for FormValue {
    fn from(value: Measurements) -> Self {
        Self::Measured(value)
    }
}
impl From<Caption> for FormValue {
    fn from(value: Caption) -> Self {
        Self::Named(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Measurements {
    pub components: [u64; 3],
    pub interval: Interval,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Caption {
    pub text: String,
}

#[derive(Debug)]
pub struct ReversedInterval;
impl std::fmt::Display for ReversedInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interval endpoints are reversed")
    }
}
impl std::error::Error for ReversedInterval {}

impl Interval {
    pub fn checked(start: u64, end: u64) -> Result<Self, ReversedInterval> {
        if start > end {
            return Err(ReversedInterval);
        }
        Ok(Self { start, end })
    }
}

pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<Output>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> Output,
    ) -> Output {
        use_admission(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}

impl Record<Interval> for Bindings {
    fn record() -> impl Shape<Interval, Self> {
        field("start", |value: &Interval| value.start)
            .and("end", |value: &Interval| value.end)
            .build(|start, end, _: &mut ValueDecodeAdmission<'_>| Interval::checked(start, end))
    }
}
impl Record<Measurements> for Bindings {
    fn record() -> impl Shape<Measurements, Self> {
        field("components", |value: &Measurements| value.components)
            .and("interval", |value: &Measurements| value.interval.clone())
            .build(|components, interval, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Measurements {
                    components,
                    interval,
                })
            })
    }
}
impl Record<Caption> for Bindings {
    fn record() -> impl Shape<Caption, Self> {
        field("text", |value: &Caption| value.text.clone()).build(
            |text, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Caption { text })
            },
        )
    }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub UnitInput for () { identity: "fixture.operation.input.v1" });
pub struct SetMetric;
impl OperationCreates<SetMetric> for Item {}
impl<S> ApplicationOperationMarkerIdentity<S> for SetMetric {
    type InputBinding = UnitInput;
    const IDENTIFIER: &'static str = "set_metric";
}
impl<S> Registration<S> for SetMetric {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let operation = ApplicationOperationRef::<S, Self, ()>::from_declaration();
        let selected: RecordField<_, _, u64> =
            record_field(Fixture, Item, Metrics, "components[1]")?;
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_record_field(operation, selected.clone())
            .operation_write_record_field(operation, selected))
    }
}

pub trait BindingSample {
    fn value(&self) -> &dyn Any;
    fn equals(&self, decoded: &dyn Any) -> bool;
    fn decode(
        &self,
        values: &std::collections::BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Box<dyn Any>, String>;
}
struct Sample<T> {
    value: T,
    decode: fn(
        &std::collections::BTreeMap<String, ApplicationValue>,
        &str,
        &mut ValueDecodeAdmission<'_>,
    ) -> Result<T, String>,
}
impl<T: Any + Eq> BindingSample for Sample<T> {
    fn value(&self) -> &dyn Any {
        &self.value
    }
    fn equals(&self, decoded: &dyn Any) -> bool {
        decoded.downcast_ref::<T>() == Some(&self.value)
    }
    fn decode(
        &self,
        values: &std::collections::BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Box<dyn Any>, String> {
        Ok(Box::new((self.decode)(values, prefix, work)?))
    }
}
pub(crate) fn row_sample<T: Any + Eq>(value: T) -> Box<dyn BindingSample>
where
    Bindings: Record<T>,
{
    Box::new(Sample {
        value,
        decode: |values, prefix, work| {
            <Bindings as Record<T>>::record()
                .decode(values, prefix, work)
                .map_err(|cause| format!("{cause:?}"))
        },
    })
}
pub(crate) fn scalar_sample<T: Any + Eq>(value: T) -> Box<dyn BindingSample>
where
    Bindings: Scalar<T>,
{
    Box::new(Sample {
        value,
        decode: |values, prefix, work| {
            let value = values
                .get(prefix)
                .ok_or_else(|| format!("missing scalar sample at {prefix}"))?;
            let value = Bindings::with_admission(work, |admission| <<Bindings as Scalar<T>>::Binding as ApplicationReadableScalarValueBinding>::decode_admitted(value, admission))
                .map_err(|cause| format!("{cause:?}"))?;
            <<Bindings as Scalar<T>>::Binding as ApplicationScalarValueBinding>::validate(&value)
                .map_err(|cause| format!("{cause:?}"))?;
            Ok(value)
        },
    })
}

pub(crate) fn sample_with<T: Any + Eq>(
    value: T,
    decode: fn(
        &std::collections::BTreeMap<String, ApplicationValue>,
        &str,
        &mut ValueDecodeAdmission<'_>,
    ) -> Result<T, String>,
) -> Box<dyn BindingSample> {
    Box::new(Sample { value, decode })
}

pub fn samples(binding: &RecordBinding) -> Vec<Box<dyn BindingSample>> {
    if binding.binding_set() == TypeId::of::<super::fragment_context_fixture::ContextBindings>() {
        return super::fragment_context_fixture::samples(binding.value_type());
    }
    let value = binding.value_type();
    if let Some(samples) = super::fragment_auth_fixture::samples(value) {
        return samples;
    }
    if value == TypeId::of::<u64>() {
        return vec![scalar_sample(0u64), scalar_sample(u64::MAX)];
    }
    if value == TypeId::of::<String>() {
        return vec![scalar_sample("measured ?".to_owned())];
    }
    if value == TypeId::of::<Interval>() {
        return vec![row_sample(Interval::checked(3, 17).unwrap())];
    }
    if value == TypeId::of::<Measurements>() {
        return vec![row_sample(Measurements {
            components: [2, 5, 11],
            interval: Interval::checked(7, 19).unwrap(),
        })];
    }
    if value == TypeId::of::<Caption>() {
        return vec![row_sample(Caption {
            text: "named".to_owned(),
        })];
    }
    Vec::new()
}

impl ApplicationSchema for Fixture {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
