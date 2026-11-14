//! An independent optional native field makes absence-only capture observable.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{
    self, Aspect, AspectSelection, FieldSelection, Shape,
};
pub struct Annotation;
impl Aspect for Annotation {
    const NAME: &'static str = "SparseAnnotation";
    type Row = AnnotationRow;
}
impl Annotation {
    pub fn reference() -> AspectSelection<PatternModel, Record, Self> {
        fragment::aspect(PatternModel, Record, Self)
    }
}
pub struct AnnotationRow {
    pub value: u64,
}
impl fragment::Record<AnnotationRow> for PatternModelBindings {
    fn record() -> impl Shape<AnnotationRow, Self> {
        fragment::field("Value", |row: &AnnotationRow| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AnnotationRow { value })
            },
        )
    }
}
pub type AnnotationValue = FieldSelection<PatternModel, Record, Annotation, u64>;
