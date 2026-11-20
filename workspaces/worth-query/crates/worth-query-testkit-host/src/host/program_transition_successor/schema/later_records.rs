//! B's additive record is separate from the unchanged shared Home contracts.
use worth_query_decl::facade::application_schema::{fragment::*, *};
pub struct LaterRecords;
impl Fragment for LaterRecords {
    const NAME: &'static str = "later_records";
    type Bindings = LaterBindings;
    type Members = (LaterRecord,);
}
pub struct LaterBindings;
impl BindingSet for LaterBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for LaterBindings {
    type Binding = StringApplicationValueBinding;
}
pub struct LaterRecord;
impl Entity for LaterRecord {
    const NAME: &'static str = "LaterRecord";
    type Aspects = (LaterIdentity,);
    type Relations = ();
}
pub struct LaterIdentity;
impl Aspect for LaterIdentity {
    const NAME: &'static str = "LaterIdentity";
    type Row = LaterIdentityRow;
}
pub struct LaterIdentityRow {
    pub label: String,
}
impl Record<LaterIdentityRow> for LaterBindings {
    fn record() -> impl Shape<LaterIdentityRow, Self> {
        field("Label", |row: &LaterIdentityRow| row.label.clone())
            .equality_queryable(true)
            .build(|label, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(LaterIdentityRow { label })
            })
    }
}
