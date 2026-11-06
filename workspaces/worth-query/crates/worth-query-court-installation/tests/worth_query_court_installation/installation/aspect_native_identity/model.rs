//! Generic source/realization records use the same array lowering as applications.
use worth_query_declaration::facade::application_schema::fragment::{self};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;
mod records;

pub struct Model<const RECORDS: usize, const FIELDS: usize, const REVERSE: bool = false>;
pub struct ApplyPreserve;
const RECORD_NAMES: [&str; 18] = [
    "Source",
    "Realization",
    "OutputA",
    "OutputB",
    "OutputC",
    "OutputD",
    "OutputE",
    "OutputF",
    "OutputG",
    "OutputH",
    "OutputI",
    "OutputJ",
    "OutputK",
    "OutputL",
    "OutputM",
    "OutputN",
    "OutputO",
    "OutputP",
];
const RELATION_NAMES: [&str; 18] = [
    "SourceOwner",
    "RealizationOwner",
    "OwnerA",
    "OwnerB",
    "OwnerC",
    "OwnerD",
    "OwnerE",
    "OwnerF",
    "OwnerG",
    "OwnerH",
    "OwnerI",
    "OwnerJ",
    "OwnerK",
    "OwnerL",
    "OwnerM",
    "OwnerN",
    "OwnerO",
    "OwnerP",
];

worth_query_declaration::worth_query_structured_value_binding!(pub InputBinding for () { identity: "worth.query.court.aspect-native-input.v1" });
impl<S> ApplicationOperationMarkerIdentity<S> for ApplyPreserve {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "apply-preserve";
}
pub fn operation<S>() -> ApplicationOperationRef<S, ApplyPreserve, ()> {
    ApplicationOperationRef::from_declaration()
}
impl<const R: usize, const F: usize, const REVERSE: bool> ApplicationSchema
    for Model<R, F, REVERSE>
{
    const OWNER: &'static str = "worth.query.court.aspect-native";
    const NAME: &'static str = "SourceAndRealization";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration::<R, F, REVERSE>()
    }
}
pub fn declaration<const R: usize, const F: usize, const REVERSE: bool>(
) -> Result<ApplicationSchemaDeclaration<Model<R, F, REVERSE>>, ApplicationSchemaDeclarationDenial>
{
    assert!(matches!(R, 2 | 4 | 8 | 18));
    assert!((1..=16).contains(&F));
    let schema = records::builder::<Model<R, F, REVERSE>, F>(R, REVERSE)?;
    schema
        .operation(
            operation()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
}
pub fn install<const R: usize, const F: usize>() -> Result<
    WorthQueryInstalledApplicationOperation<Model<R, F>, ApplyPreserve, ()>,
    WorthQueryApplicationOperationInstallationDenial,
> {
    let declaration = Model::<R, F>::declaration().expect("valid aspect-native declaration");
    let index = index(&declaration);
    let schema = index
        .bind_application_schema(declaration)
        .expect("installed native catalog");
    schema.installed_operation(operation())
}

pub fn index<const R: usize, const F: usize, const REVERSE: bool>(
    declaration: &ApplicationSchemaDeclaration<Model<R, F, REVERSE>>,
) -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Model::<R, F, REVERSE>::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("valid portable package");
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .expect("admitted package");
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("installed package index")
}
