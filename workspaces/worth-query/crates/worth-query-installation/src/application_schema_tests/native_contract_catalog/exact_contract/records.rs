//! One record model owns field lowering and reusable scalar bindings.

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub(super) struct ExactRecords;
impl fragment::Fragment for ExactRecords {
    const NAME: &'static str = "exact_records";
    type Bindings = ExactRecordsBindings;
    type Members = (ExactEntity,);
}
pub(super) struct ExactRecordsBindings;
impl fragment::BindingSet for ExactRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for ExactRecordsBindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for ExactRecordsBindings {
    type Binding =
        worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<bool> for ExactRecordsBindings {
    type Binding = worth_query_declaration::facade::application_schema::BoolApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
pub(super) struct ExactEntity;
impl fragment::Entity for ExactEntity {
    const NAME: &'static str = "ExactEntity";
    type Aspects = (ExactAspect,);
    type Relations = ();
}

pub(super) struct ExactAspect;
impl fragment::Aspect for ExactAspect {
    const NAME: &'static str = "ExactAspect";
    type Row = ExactAspectValue;
}

pub(super) struct ExactAspectValue {
    pub required_count: u64,
    pub optional_label: Option<String>,
    pub required_active: bool,
}
impl fragment::Record<ExactAspectValue> for ExactRecordsBindings {
    fn record() -> impl fragment::Shape<ExactAspectValue, Self> {
        fragment::field("RequiredCount", |row: &ExactAspectValue| row.required_count)
            .and("OptionalLabel", |row: &ExactAspectValue| {
                row.optional_label.clone()
            })
            .and("RequiredActive", |row: &ExactAspectValue| {
                row.required_active
            })
            .build(
                |required_count,
                 optional_label,
                 required_active,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(ExactAspectValue {
                        required_count,
                        optional_label,
                        required_active,
                    })
                },
            )
    }
}
pub(super) type RequiredCount = FieldSelection<ExactRecords, ExactEntity, ExactAspect, u64>;
