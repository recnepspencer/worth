//! Equal portable records remain equal when their Rust binding sets and row types differ.
pub use worth_query_decl::facade::application_schema::{fragment::*, *};
pub struct Source;
pub struct Target;
pub struct SourceBindings;
pub struct TargetBindings;
pub struct SourceItem;
pub struct TargetItem;
pub struct SourceFacts;
pub struct TargetFacts;
pub struct SourceRow(pub u64);
pub struct TargetRow(pub u64);
impl Fragment for Source {
    const NAME: &'static str = "shape_fingerprint";
    type Bindings = SourceBindings;
    type Members = (SourceItem,);
}
impl Fragment for Target {
    const NAME: &'static str = "shape_fingerprint";
    type Bindings = TargetBindings;
    type Members = (TargetItem,);
}
impl ApplicationSchema for Source {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
impl ApplicationSchema for Target {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
impl Entity for SourceItem {
    const NAME: &'static str = "item";
    type Aspects = (SourceFacts,);
    type Relations = ();
}
impl Entity for TargetItem {
    const NAME: &'static str = "item";
    type Aspects = (TargetFacts,);
    type Relations = ();
}
impl Aspect for SourceFacts {
    const NAME: &'static str = "facts";
    type Row = SourceRow;
}
impl Aspect for TargetFacts {
    const NAME: &'static str = "facts";
    type Row = TargetRow;
}
impl BindingSet for SourceBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl BindingSet for TargetBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for SourceBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<u64> for TargetBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Record<SourceRow> for SourceBindings {
    fn record() -> impl Shape<SourceRow, Self> {
        field("value", |row: &SourceRow| row.0).build(|value, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(SourceRow(value))
        })
    }
}
impl Record<TargetRow> for TargetBindings {
    fn record() -> impl Shape<TargetRow, Self> {
        field("value", |row: &TargetRow| row.0).build(|value, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(TargetRow(value))
        })
    }
}
