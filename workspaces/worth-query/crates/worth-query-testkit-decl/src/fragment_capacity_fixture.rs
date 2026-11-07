//! A field with a finite codec capacity, used by real installation seeding courts.
use super::fragment_fixture::Item;
use worth_query_decl::facade::application_schema::{fragment::*, *};

pub struct Limited;
pub struct Bindings;
pub struct Text;
#[derive(Debug, Eq, PartialEq)]
pub struct Row {
    pub text: String,
}
impl Fragment for Limited {
    const NAME: &'static str = "limited";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Option<Text>,)>,);
}
impl Aspect for Text {
    const NAME: &'static str = "text";
    type Row = Row;
}
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub TextBinding for String {
    identity: "fixture.limited.text.v1", scalar: String,
    canonical: ValueDecodeBounds::new(64, 3, 1, 1),
});
impl Scalar<String> for Bindings {
    type Binding = TextBinding;
}
impl Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("text", |row: &Row| row.text.clone()).build(
            |text, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { text })
            },
        )
    }
}

pub(crate) fn samples(
    value: std::any::TypeId,
) -> Vec<Box<dyn super::fragment_fixture::BindingSample>> {
    use super::fragment_fixture::sample_with;
    if value == std::any::TypeId::of::<String>() {
        return vec![sample_with("ok".to_owned(), |values, prefix, work| {
            let value = values
                .get(prefix)
                .ok_or_else(|| format!("missing scalar {prefix}"))?;
            TextBinding::decode_admitted(value, work).map_err(|cause| format!("{cause:?}"))
        })];
    }
    if value == std::any::TypeId::of::<Row>() {
        return vec![sample_with(
            Row {
                text: "ok".to_owned(),
            },
            |values, prefix, work| {
                <Bindings as Record<Row>>::record()
                    .decode(values, prefix, work)
                    .map_err(|cause| format!("{cause:?}"))
            },
        )];
    }
    Vec::new()
}
