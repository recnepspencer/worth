//! An owner carries a mark or none, so a sparse inverse must restore absence.
use super::*;
use worth_query_decl::facade::application_schema::fragment::*;
pub struct SourceMark;
impl Aspect for SourceMark {
    const NAME: &'static str = "SourceMark";
    type Row = SourceMarkValue;
}
impl SourceMark {
    pub fn reference() -> AspectSelection<SourceFeature, Source, Self> {
        aspect(SourceFeature, Source, Self)
    }
}
pub struct SourceMarkValue {
    pub mark: u64,
}
impl Record<SourceMarkValue> for SourceModelBindings {
    fn record() -> impl Shape<SourceMarkValue, Self> {
        field("OwnerMark", |row: &SourceMarkValue| row.mark).build(
            |mark, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SourceMarkValue { mark })
            },
        )
    }
}
pub type OwnerMark = FieldSelection<SourceFeature, Source, SourceMark, u64>;
