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
    const EQUALITY_QUERYABLE: bool = true;
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
        fragment::field("values", |r: &Row<F>| r.0).build(
            |values, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row(values))
            },
        )
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
impl<const F: usize> Entity for Record<1, F> {
    const NAME: &'static str = RECORD_NAMES[1];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<1, F>,);
}
impl<const F: usize> Entity for Record<2, F> {
    const NAME: &'static str = RECORD_NAMES[2];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<2, F>,);
}
impl<const F: usize> Entity for Record<3, F> {
    const NAME: &'static str = RECORD_NAMES[3];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<3, F>,);
}
impl<const F: usize> Entity for Record<4, F> {
    const NAME: &'static str = RECORD_NAMES[4];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<4, F>,);
}
impl<const F: usize> Entity for Record<5, F> {
    const NAME: &'static str = RECORD_NAMES[5];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<5, F>,);
}
impl<const F: usize> Entity for Record<6, F> {
    const NAME: &'static str = RECORD_NAMES[6];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<6, F>,);
}
impl<const F: usize> Entity for Record<7, F> {
    const NAME: &'static str = RECORD_NAMES[7];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<7, F>,);
}
impl<const F: usize> Entity for Record<8, F> {
    const NAME: &'static str = RECORD_NAMES[8];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<8, F>,);
}
impl<const F: usize> Entity for Record<9, F> {
    const NAME: &'static str = RECORD_NAMES[9];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<9, F>,);
}
impl<const F: usize> Entity for Record<10, F> {
    const NAME: &'static str = RECORD_NAMES[10];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<10, F>,);
}
impl<const F: usize> Entity for Record<11, F> {
    const NAME: &'static str = RECORD_NAMES[11];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<11, F>,);
}
impl<const F: usize> Entity for Record<12, F> {
    const NAME: &'static str = RECORD_NAMES[12];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<12, F>,);
}
impl<const F: usize> Entity for Record<13, F> {
    const NAME: &'static str = RECORD_NAMES[13];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<13, F>,);
}
impl<const F: usize> Entity for Record<14, F> {
    const NAME: &'static str = RECORD_NAMES[14];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<14, F>,);
}
impl<const F: usize> Entity for Record<15, F> {
    const NAME: &'static str = RECORD_NAMES[15];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<15, F>,);
}
impl<const F: usize> Entity for Record<16, F> {
    const NAME: &'static str = RECORD_NAMES[16];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<16, F>,);
}
impl<const F: usize> Entity for Record<17, F> {
    const NAME: &'static str = RECORD_NAMES[17];
    type Aspects = (Facts<F>,);
    type Relations = (Owner<17, F>,);
}

mod count_18;
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
        _ => unreachable!("fixture range checked by caller"),
    }
}

impl<const I: usize, const F: usize> OperationCreates<ApplyPreserve> for Record<I, F> {}
impl<const I: usize, const F: usize> OperationReads<ApplyPreserve> for Owner<I, F> {}
impl<const I: usize, const F: usize> OperationLinks<ApplyPreserve> for Owner<I, F> {}
