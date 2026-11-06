//! The counted application owns its body header and count records.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct QuoteRecords;
impl Fragment for QuoteRecords {
    const NAME: &'static str = "quote_records";
    type Bindings = TopologyRecordsBindings;
    type Members = (ExternalPrincipalMapping, Body);
}
pub struct Body;
impl Entity for Body {
    const NAME: &'static str = "Body";
    type Aspects = (PlanarPosition, QuoteSourceFacts, QuoteOutputFacts);
    type Relations = ();
}
impl Body {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationEntityRef<S, Self> {
        entity(QuoteRecords, Self)
    }
}
pub struct PlanarPosition;
impl Aspect for PlanarPosition {
    const NAME: &'static str = "PlanarPosition";
    type Row = Header;
}
pub struct Header(String);
impl fragment::Record<Header> for TopologyRecordsBindings {
    fn record() -> impl Shape<Header, Self> {
        field("BodyKey", |row: &Header| row.0.clone()).build(
            |key, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(Header(key)),
        )
    }
}
pub type BodyKey = FieldSelection<QuoteRecords, Body, PlanarPosition, String>;
pub type PlanarReadScope =
    worth_query_decl::facade::application_query::ApplicationQueryFieldScope<BodyKey>;
pub struct QuoteSourceFacts;
impl Aspect for QuoteSourceFacts {
    const NAME: &'static str = "QuoteSourceFacts";
    type Row = Requested;
}
pub struct Requested(Count);
impl fragment::Record<Requested> for TopologyRecordsBindings {
    fn record() -> impl Shape<Requested, Self> {
        field("RequestedRows", |row: &Requested| row.0).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Requested(value))
            },
        )
    }
}
pub struct QuoteOutputFacts;
impl Aspect for QuoteOutputFacts {
    const NAME: &'static str = "QuoteOutputFacts";
    type Row = Counted;
}
pub struct Counted(Count);
impl fragment::Record<Counted> for TopologyRecordsBindings {
    fn record() -> impl Shape<Counted, Self> {
        field("CountedTotal", |row: &Counted| row.0).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Counted(value))
            },
        )
    }
}
pub type RequestedRows = FieldSelection<QuoteRecords, Body, QuoteSourceFacts, Count>;
pub type CountedTotal = FieldSelection<QuoteRecords, Body, QuoteOutputFacts, Count>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Count(u64);
impl From<u64> for Count {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<Count> for u64 {
    fn from(value: Count) -> Self {
        value.0
    }
}
pub struct CountBinding;
impl ApplicationScalarValueBinding for CountBinding {
    type Value = Count;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "quote-count.count";
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        worth_query_decl::facade::typed::ScalarAspectType::UInt64;
    const SUPPORTS_ADMITTED_DECODE: bool = U64ApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        U64ApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &Count) -> Result<(), ApplicationValueValidationDenial> {
        U64ApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &Count) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        U64ApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for CountBinding {
    fn decode(raw: &ApplicationValue) -> Result<Count, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode(raw).map(Count)
    }
    fn decode_admitted(
        raw: &ApplicationValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Count, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode_admitted(raw, work).map(Count)
    }
}
impl Scalar<Count> for TopologyRecordsBindings {
    type Binding = CountBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct PlanarAnchorOutput<S>(std::marker::PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> WorthQueryApplicationOutputRole for PlanarAnchorOutput<S> {
    type Schema = S;
    type Contract = crate::PlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str =
        <crate::PlanarAnchorOutput<S> as WorthQueryApplicationOutputRole>::NAME;
}

use worth_query_decl::facade::application_operation::{
    WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};
