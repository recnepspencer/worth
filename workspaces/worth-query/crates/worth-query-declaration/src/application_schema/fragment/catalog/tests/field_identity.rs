//! Rust marker spelling is not field identity; its declared namespace and codec are.
use super::*;
use crate::application_schema::*;

struct RenamedSuite;
impl Fragment for RenamedSuite {
    const NAME: &'static str = Suite::NAME;
    type Bindings = Bindings;
    type Members = <Suite as Fragment>::Members;
}
struct OtherNamespace;
impl Fragment for OtherNamespace {
    const NAME: &'static str = "other";
    type Bindings = Bindings;
    type Members = <Suite as Fragment>::Members;
}
struct OtherCodec;
impl ApplicationScalarValueBinding for OtherCodec {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "tests.other-u64";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        worth_foundational::facade::ScalarAspectType::UInt64;
    fn validate(value: &u64) -> Result<(), ApplicationValueValidationDenial> {
        <U64ApplicationValueBinding as ApplicationScalarValueBinding>::validate(value)
    }
    fn encode(value: &u64) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        <U64ApplicationValueBinding as ApplicationScalarValueBinding>::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for OtherCodec {
    fn decode(value: &ApplicationValue) -> Result<u64, ApplicationValueDecodeDenial> {
        <U64ApplicationValueBinding as ApplicationReadableScalarValueBinding>::decode(value)
    }
}
struct OtherBindings;
impl BindingSet for OtherBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for OtherBindings {
    type Binding = OtherCodec;
}
impl Record<CountValue> for OtherBindings {
    fn record() -> impl Shape<CountValue, Self> {
        field("count", |value: &CountValue| value.count).build(
            |count, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CountValue { count })
            },
        )
    }
}
struct OtherMeaning;
impl Fragment for OtherMeaning {
    const NAME: &'static str = Suite::NAME;
    type Bindings = OtherBindings;
    type Members = <Suite as Fragment>::Members;
}
#[test]
fn a_named_field_survives_marker_renaming_without_admitting_another_namespace_or_codec() {
    let mut catalog = FragmentCatalog::default();
    catalog.register::<Suite>(Suite::NAME).unwrap();
    catalog.entity::<Item>("suite.item").unwrap();
    let columns = <Bindings as Record<u64>>::record().columns("").unwrap();
    catalog
        .aspect::<Bindings, Count>("suite.item", "item", &columns, false)
        .unwrap();
    let original = FieldSelection::<Suite, Item, Count, u64>::at("value")
        .installed(&catalog)
        .unwrap();
    let renamed = FieldSelection::<RenamedSuite, Item, Count, u64>::at("value")
        .installed(&catalog)
        .unwrap();
    assert_eq!(renamed, original);
    assert!(
        catalog.fragment_type::<RenamedSuite>().is_none(),
        "a field selection grants no fragment view"
    );
    for refused in [
        FieldSelection::<OtherNamespace, Item, Count, u64>::at("value").installed(&catalog),
        FieldSelection::<OtherMeaning, Item, Count, u64>::at("value").installed(&catalog),
    ] {
        assert!(matches!(refused, Err(Denial::InvalidRecordField { .. })));
    }
}
