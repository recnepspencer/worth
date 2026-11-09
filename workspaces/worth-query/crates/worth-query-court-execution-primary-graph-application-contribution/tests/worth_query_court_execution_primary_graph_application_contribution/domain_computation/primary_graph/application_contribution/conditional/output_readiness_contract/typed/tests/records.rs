//! Each schema variant owns its genuine row; field pairing is shared once.
use super::{schema, Binding, ChangedBinding, ChangedPayload, Payload, TwinBinding};
use schema::fragment::{self, *};
pub(super) struct Model<const N: u8>;
pub(super) struct Bindings<const N: u8>;
pub(super) struct Node<const N: u8>;
pub(super) struct Publication<const N: u8>;
pub(super) struct Identity;
pub(super) struct KeyRow(String);
pub(super) struct PublicationRow<T>(T, Option<String>);
pub(super) struct MissingRow(Option<String>);
impl<const N: u8> Fragment for Model<N>
where
    Publication<N>: Aspect,
{
    const NAME: &'static str = "readiness";
    type Bindings = Bindings<N>;
    type Members = (Node<N>,);
}
impl<const N: u8> Entity for Node<N>
where
    Publication<N>: Aspect,
{
    const NAME: &'static str = "Entity";
    type Aspects = (Identity, Option<Publication<N>>);
    type Relations = ();
}
impl<const N: u8> BindingSet for Bindings<N> {
    type Work<'a> = schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl<const N: u8> Scalar<String> for Bindings<N> {
    type Binding = schema::StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<Payload> for Bindings<0> {
    type Binding = Binding;
    const WRITABLE: bool = false;
}
impl Scalar<ChangedPayload> for Bindings<1> {
    type Binding = ChangedBinding;
    const WRITABLE: bool = false;
}
impl Scalar<Payload> for Bindings<2> {
    type Binding = TwinBinding;
    const WRITABLE: bool = false;
}
impl Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = KeyRow;
}
impl Aspect for Publication<0> {
    const NAME: &'static str = "Publication";
    type Row = PublicationRow<Payload>;
}
impl Aspect for Publication<1> {
    const NAME: &'static str = "Publication";
    type Row = PublicationRow<ChangedPayload>;
}
impl Aspect for Publication<2> {
    const NAME: &'static str = "Publication";
    type Row = PublicationRow<Payload>;
}
impl Aspect for Publication<3> {
    const NAME: &'static str = "Publication";
    type Row = MissingRow;
}
impl<const N: u8> fragment::Record<KeyRow> for Bindings<N> {
    fn record() -> impl Shape<KeyRow, Self> {
        field("Key", |row: &KeyRow| row.0.clone()).build(
            |key, _: &mut <Self as BindingSet>::Work<'_>| {
                Ok::<_, std::convert::Infallible>(KeyRow(key))
            },
        )
    }
}
impl<const N: u8, T: Clone + 'static> fragment::Record<PublicationRow<T>> for Bindings<N>
where
    Self: Scalar<T>,
{
    fn record() -> impl Shape<PublicationRow<T>, Self> {
        field("Record", |row: &PublicationRow<T>| row.0.clone())
            .and("Sibling", |row: &PublicationRow<T>| row.1.clone())
            .build(|record, sibling, _: &mut <Self as BindingSet>::Work<'_>| {
                Ok::<_, std::convert::Infallible>(PublicationRow(record, sibling))
            })
    }
}
impl fragment::Record<MissingRow> for Bindings<3> {
    fn record() -> impl Shape<MissingRow, Self> {
        field("Sibling", |row: &MissingRow| row.0.clone()).build(
            |sibling, _: &mut <Self as BindingSet>::Work<'_>| {
                Ok::<_, std::convert::Infallible>(MissingRow(sibling))
            },
        )
    }
}

impl Aspect for Publication<4> {
    const NAME: &'static str = "Publication";
    type Row = u64;
}
impl Scalar<u64> for Bindings<4> {
    type Binding = schema::U64ApplicationValueBinding;
}
