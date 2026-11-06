//! A declaration cannot silently change a field's identity or uniqueness meaning.
use worth_query_testkit_decl::fragment_fixture::*;
struct InvalidBindings;
struct Pair([u64; 2]);
impl BindingSet for InvalidBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        decode: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        decode(work)
    }
}
impl Scalar<u64> for InvalidBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Record<Pair> for InvalidBindings {
    fn record() -> impl Shape<Pair, Self> {
        field("pair", |value: &Pair| value.0).unique().build(
            |pair, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(Pair(pair)),
        )
    }
}
impl Record<Interval> for InvalidBindings {
    fn record() -> impl Shape<Interval, Self> {
        field("value", |value: &Interval| value.start)
            .and("value", |value: &Interval| value.end)
            .build(|start, end, _: &mut ValueDecodeAdmission<'_>| Interval::checked(start, end))
    }
}
#[test]
fn uniqueness_cannot_turn_into_independent_constraints_on_array_elements() {
    let denied = <InvalidBindings as Record<Pair>>::record()
        .encode(&Pair([3, 7]), "fixture.item.pair")
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.item.pair.pair");
    assert!(matches!(
        denied.kind(),
        RecordRefusalKind::UniqueNeedsScalar
    ));
}
#[test]
fn duplicate_field_names_refuse_before_an_encoder_can_overwrite_a_value() {
    let denied = <InvalidBindings as Record<Interval>>::record()
        .encode(&Interval::checked(3, 7).unwrap(), "fixture.item.interval")
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.item.interval.value");
    assert!(matches!(denied.kind(), RecordRefusalKind::DuplicateField));
}
