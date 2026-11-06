//! Shape changes derive revisions; no fixture chooses a revision number.
use super::schema;
use schema::fragment::{self, *};
pub struct Model<const N: u8>;
pub struct Bindings;
#[derive(Clone, Copy)]
pub struct Output<const N: u8>;
pub struct Values<const N: u8>;
pub struct One(u64);
pub struct Pair(u64, u64);
pub struct Sibling(u64);
impl<const N: u8> Fragment for Model<N>
where
    Values<N>: Aspect,
{
    const NAME: &'static str = "equivalence";
    type Bindings = Bindings;
    type Members = (Output<N>,);
}
impl<const N: u8> Entity for Output<N>
where
    Values<N>: Aspect,
{
    const NAME: &'static str = "Output";
    type Aspects = (Option<Values<N>>,);
    type Relations = ();
}
impl BindingSet for Bindings {
    type Work<'a> = schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = schema::U64ApplicationValueBinding;
}
impl Aspect for Values<0> {
    const NAME: &'static str = "Values";
    type Row = One;
}
impl Aspect for Values<1> {
    const NAME: &'static str = "Values";
    type Row = u64;
}
impl Aspect for Values<2> {
    const NAME: &'static str = "Values";
    type Row = Pair;
}
impl Aspect for Values<3> {
    const NAME: &'static str = "Values";
    type Row = Sibling;
}
impl fragment::Record<One> for Bindings {
    fn record() -> impl Shape<One, Self> {
        field("Value", |row: &One| row.0).build(|v, _: &mut schema::ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(One(v))
        })
    }
}
impl fragment::Record<Pair> for Bindings {
    fn record() -> impl Shape<Pair, Self> {
        field("Value", |row: &Pair| row.0)
            .and("Sibling", |row: &Pair| row.1)
            .build(|a, b, _: &mut schema::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Pair(a, b))
            })
    }
}
impl fragment::Record<Sibling> for Bindings {
    fn record() -> impl Shape<Sibling, Self> {
        field("Sibling", |row: &Sibling| row.0).build(
            |v, _: &mut schema::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Sibling(v))
            },
        )
    }
}
