pub use worth_foundational::facade::{AspectValue, ScalarAspectType};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationValueEncodeDenial,
};
pub use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub use worth_query_declaration::facade::application_schema::fragment::{
    record_field, RecordField,
};
use worth_query_declaration::facade::application_schema::{fragment::*, *};

pub struct BindingSchema;
impl Fragment for BindingSchema {
    const NAME: &'static str = "BindingSchema";
    type Bindings = Bindings;
    type Members = (Component,);
}
impl ApplicationSchema for BindingSchema {
    const OWNER: &'static str = "worth.tests.value-binding-installation";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct Component;
impl Entity for Component {
    const NAME: &'static str = "Component";
    type Aspects = (Measurement,);
    type Relations = ();
}
pub struct Measurement;
impl Aspect for Measurement {
    const NAME: &'static str = "Measurement";
    type Row = u64;
}

pub fn installed_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        BindingSchema::OWNER,
        1,
        0,
    ))
    .application_schema(BindingSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}
