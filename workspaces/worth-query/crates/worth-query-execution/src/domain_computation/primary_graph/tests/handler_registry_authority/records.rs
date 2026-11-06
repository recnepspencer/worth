//! One record model owns field lowering and reusable scalar bindings.
use worth_query_declaration::facade::application_schema::{
    fragment::{self},
    *,
};
pub struct RegistryRecords;
impl fragment::Fragment for RegistryRecords {
    const NAME: &'static str = "registry_records";
    type Bindings = RegistryRecordsBindings;
    type Members = (InstalledEntity,);
}
pub struct RegistryRecordsBindings;
impl fragment::BindingSet for RegistryRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for RegistryRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
pub struct InstalledEntity;
impl fragment::Entity for InstalledEntity {
    const NAME: &'static str = "InstalledEntity";
    type Aspects = (InstalledAspect,);
    type Relations = ();
}

pub struct InstalledAspect;
impl fragment::Aspect for InstalledAspect {
    const NAME: &'static str = "InstalledAspect";
    type Row = InstalledAspectValue;
}

pub struct InstalledAspectValue {
    pub installed_field: u64,
}
impl fragment::Record<InstalledAspectValue> for RegistryRecordsBindings {
    fn record() -> impl fragment::Shape<InstalledAspectValue, Self> {
        fragment::field("InstalledField", |row: &InstalledAspectValue| {
            row.installed_field
        })
        .build(|installed_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(InstalledAspectValue { installed_field })
        })
    }
}
