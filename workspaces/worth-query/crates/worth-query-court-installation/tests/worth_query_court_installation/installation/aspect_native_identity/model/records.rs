//! Binding, row and entity ownership for the identity fixture.
use super::*;
use fragment::{Aspect, BindingSet, Entity, Fragment, Relation};
pub struct Records<const N: usize, const F: usize, const REVERSE: bool>;
pub struct Set;
impl BindingSet for Set {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        w: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(w)
    }
}
impl fragment::Scalar<u64> for Set {
    type Binding = U64ApplicationValueBinding;
}
pub struct Record<const I: usize, const F: usize>;
pub struct Facts<const F: usize>;
pub struct Row<const F: usize>([u64; F]);
impl<const F: usize> Aspect for Facts<F> {
    const NAME: &'static str = "Facts";
    type Row = Row<F>;
}
impl<const F: usize> fragment::Record<Row<F>> for Set {
    fn record() -> impl fragment::Shape<Row<F>, Self> {
        fragment::field("values", |r: &Row<F>| r.0)
            .equality_queryable(true)
            .build(|values, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row(values))
            })
    }
}
pub struct Owner<const I: usize, const F: usize>;
impl<const I: usize, const F: usize> Relation for Owner<I, F> {
    const NAME: &'static str = RELATION_NAMES[I];
    type To = Record<0, F>;
}
impl<const F: usize> Entity for Record<0, F> {
    const NAME: &'static str = RECORD_NAMES[0];
    type Aspects = (Facts<F>,);
    type Relations = ();
}
macro_rules! record_entities {
    ($($i:literal),*) => { $(
        impl<const F: usize> Entity for Record<$i, F> {
            const NAME: &'static str = RECORD_NAMES[$i];
            type Aspects = (Facts<F>,);
            type Relations = (Owner<$i, F>,);
        }
    )* };
}
record_entities!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
);

mod count_18;
mod count_40;
mod count_8;
mod count_small;
pub fn builder<S: ApplicationSchema, const F: usize>(
    n: usize,
    reverse: bool,
) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
    match (n, reverse) {
        (2, false) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<2, F, false>),
        (2, true) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<2, F, true>),
        (4, false) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<4, F, false>),
        (4, true) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<4, F, true>),
        (8, false) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<8, F, false>),
        (8, true) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<8, F, true>),
        (18, false) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<18, F, false>),
        (18, true) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<18, F, true>),
        (40, false) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<40, F, false>),
        (40, true) => ApplicationSchemaDeclarationBuilder::from_fragment(Records::<40, F, true>),
        _ => unreachable!("fixture range checked by caller"),
    }
}

impl<const I: usize, const F: usize> OperationCreates<ApplyPreserve> for Record<I, F> {}
impl<const I: usize, const F: usize> OperationReads<ApplyPreserve> for Owner<I, F> {}
impl<const I: usize, const F: usize> OperationLinks<ApplyPreserve> for Owner<I, F> {}
