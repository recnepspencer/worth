//! Synthetic collisions exercise the real catalog guard without a new door.
use super::*;
use crate::application_schema::{fragment::*, U64ApplicationValueBinding, ValueDecodeAdmission};
struct Suite;
struct Item;
struct Count;
struct Bindings;
impl Fragment for Suite {
    const NAME: &'static str = "suite";
    type Bindings = Bindings;
    type Members = (Item,);
}
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (Count, Option<OptionalCount>);
    type Relations = ();
}
impl Aspect for Count {
    const NAME: &'static str = "count";
    type Row = u64;
}
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
#[test]
fn a_physical_key_collision_is_a_typed_refusal() {
    let mut catalog = FragmentCatalog::default();
    catalog.register::<Suite>("suite").unwrap();
    catalog.entity::<Item>("suite.item").unwrap();
    catalog.native_aspects.insert(
        super::super::identity::native_aspect_identity("item", "count"),
        "other.meaning".to_owned(),
    );
    let columns = <Bindings as Record<u64>>::record().columns("").unwrap();
    assert!(
        matches!(catalog.aspect::<Bindings, Count>("suite.item", "item", &columns, false), Err(Denial::FragmentIdentityCollision { first, second, .. }) if first == "other.meaning" && second == "suite.item.count")
    );
}
#[test]
fn const_selectors_and_compiled_columns_share_the_same_physical_key() {
    assert_eq!(
        super::super::identity::native_aspect_key::<Item, Count>(),
        super::super::identity::native_aspect_name("item", "count")
    );
}
struct Invalid;
impl Aspect for Invalid {
    const NAME: &'static str = "invalid.name";
    type Row = u64;
}
#[test]
fn an_encoded_physical_key_cannot_hide_an_invalid_logical_name() {
    let mut catalog = FragmentCatalog::default();
    catalog.register::<Suite>("suite").unwrap();
    catalog.entity::<Item>("suite.item").unwrap();
    let columns = <Bindings as Record<u64>>::record().columns("").unwrap();
    assert!(matches!(
        catalog.aspect::<Bindings, Invalid>("suite.item", "item", &columns, false),
        Err(Denial::InvalidIdentifier)
    ));
}

struct OptionalCount;
struct CountValue {
    count: Option<u64>,
}
impl Aspect for OptionalCount {
    const NAME: &'static str = "optional_count";
    type Row = CountValue;
}
impl Record<CountValue> for Bindings {
    fn record() -> impl Shape<CountValue, Self> {
        field("count", |value: &CountValue| value.count).build(
            |count, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CountValue { count })
            },
        )
    }
}
#[test]
fn a_matching_bool_codec_cannot_select_the_sdk_presence_column() {
    use crate::application_schema::fragment::provenance::FragmentProvenance;
    let mut catalog = FragmentCatalog::default();
    catalog.register::<Suite>("suite").unwrap();
    catalog.entity::<Item>("suite.item").unwrap();
    let columns = <Bindings as Record<CountValue>>::record()
        .columns("")
        .unwrap();
    let row = catalog
        .aspect::<Bindings, OptionalCount>("suite.item", "item", &columns, true)
        .unwrap();
    let actual: Vec<_> = row
        .fields()
        .iter()
        .map(|field| field.recipe().clone())
        .collect();
    let marker = row
        .fields()
        .iter()
        .find(|field| field.internal_presence())
        .unwrap()
        .recipe()
        .clone();
    let mut provenance = FragmentProvenance::default();
    provenance.catalog = catalog;
    provenance.expect_field(marker.clone());
    assert!(
        matches!(provenance.validate(&actual), Err(Denial::InvalidRecordField { field }) if field == *marker.locus())
    );
}
#[test]
fn row_encoding_refuses_a_different_value_type_before_entering_its_codec() {
    let mut catalog = FragmentCatalog::default();
    catalog.register::<Suite>("suite").unwrap();
    catalog.entity::<Item>("suite.item").unwrap();
    let columns = <Bindings as Record<CountValue>>::record()
        .columns("")
        .unwrap();
    let row = catalog
        .aspect::<Bindings, OptionalCount>("suite.item", "item", &columns, true)
        .unwrap();
    assert_eq!(
        row.encode::<Bindings, CountValue>(&CountValue { count: None })
            .unwrap()
            .len(),
        1
    );
    assert!(matches!(
        row.encode::<Bindings, u64>(&42).unwrap_err().kind(),
        RecordRefusalKind::WrongValueType
    ));
}

#[path = "tests/nested_relations.rs"]
mod nested_relations;

mod field_identity;
