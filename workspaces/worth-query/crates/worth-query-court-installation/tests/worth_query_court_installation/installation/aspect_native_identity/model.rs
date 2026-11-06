//! Generic source/realization records use the same array lowering as applications.
use worth_query_declaration::facade::application_schema::fragment;
pub(crate) mod authorization;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;
mod records;

pub struct Model<const RECORDS: usize, const FIELDS: usize, const REVERSE: bool = false>;
pub struct ApplyPreserve;
const RECORD_NAMES: [&str; 40] = [
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
    "Output18",
    "Output19",
    "Output20",
    "Output21",
    "Output22",
    "Output23",
    "Output24",
    "Output25",
    "Output26",
    "Output27",
    "Output28",
    "Output29",
    "Output30",
    "Output31",
    "Output32",
    "Output33",
    "Output34",
    "Output35",
    "Output36",
    "Output37",
    "Output38",
    "Output39",
];
const RELATION_NAMES: [&str; 40] = [
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
    "Owner18",
    "Owner19",
    "Owner20",
    "Owner21",
    "Owner22",
    "Owner23",
    "Owner24",
    "Owner25",
    "Owner26",
    "Owner27",
    "Owner28",
    "Owner29",
    "Owner30",
    "Owner31",
    "Owner32",
    "Owner33",
    "Owner34",
    "Owner35",
    "Owner36",
    "Owner37",
    "Owner38",
    "Owner39",
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
    assert!(matches!(R, 2 | 4 | 8 | 18 | 40));
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

pub fn index<S: ApplicationSchema>(
    declaration: &ApplicationSchemaDeclaration<S>,
) -> WorthQueryInstalledPackageIndex {
    index_from(
        package(declaration)
            .validate()
            .expect("valid portable package"),
    )
}

pub fn package<S: ApplicationSchema>(
    declaration: &ApplicationSchemaDeclaration<S>,
) -> WorthQueryPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(S::OWNER, 1, 0))
        .application_schema(declaration.clone())
}

pub fn index_from(
    package: WorthQueryValidatedPortableDomainPackage,
) -> WorthQueryInstalledPackageIndex {
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
