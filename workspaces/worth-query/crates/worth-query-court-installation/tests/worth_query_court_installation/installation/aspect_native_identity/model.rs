//! Generic source/realization/output records; no product or geometry vocabulary.
pub(crate) mod authorization;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;

pub struct Model<const RECORDS: usize, const FIELDS: usize>;
pub struct ApplyPreserve;
pub struct Record<const I: usize>;
pub struct Facts<const I: usize>;
pub struct Scalar<const F: usize>;
pub struct Owner<const I: usize>;

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
const FIELD_NAMES: [&str; 16] = [
    "Identity",
    "Position",
    "Revision",
    "InputStamp",
    "PreviousValue",
    "State",
    "Sequence",
    "Weight",
    "Receipt",
    "Category",
    "Label",
    "Minimum",
    "Maximum",
    "Count",
    "Generation",
    "Status",
];

worth_query_declaration::worth_query_structured_value_binding!(pub InputBinding for () {
    identity: "worth.query.court.aspect-native-input.v1"
});
impl<S> ApplicationOperationMarkerIdentity<S> for ApplyPreserve {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "apply-preserve";
}
impl<S, const I: usize> ApplicationEntityMarkerIdentity<S> for Record<I> {
    const IDENTIFIER: &'static str = RECORD_NAMES[I];
}
impl<S, const I: usize> ApplicationAspectMarkerIdentity<S, Record<I>> for Facts<I> {
    const IDENTIFIER: &'static str = "Facts";
    const ASPECT_IDENTITY: AspectIdentity = AspectIdentity(0x371000 + I as u64);
    const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(1);
}
impl<const F: usize> DeclaredApplicationFieldValue for Scalar<F> {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}
impl<S, const I: usize, const F: usize> ApplicationFieldMarkerIdentity<S, Record<I>, Facts<I>>
    for Scalar<F>
{
    const IDENTIFIER: &'static str = FIELD_NAMES[F];
}
impl<const F: usize> OperationReads<ApplyPreserve> for Scalar<F> {}
impl<const F: usize> OperationWrites<ApplyPreserve> for Scalar<F> {}
impl<const I: usize> OperationCreates<ApplyPreserve> for Record<I> {}
impl<const I: usize> OperationReads<ApplyPreserve> for Owner<I> {}
impl<const I: usize> OperationLinks<ApplyPreserve> for Owner<I> {}

pub fn operation<S>() -> ApplicationOperationRef<S, ApplyPreserve, ()> {
    ApplicationOperationRef::from_declaration()
}
fn entity<S, const I: usize>() -> ApplicationEntityRef<S, Record<I>> {
    ApplicationEntityRef::from_schema_identifier(RECORD_NAMES[I])
}
fn field<S, const I: usize, const F: usize>(
) -> ApplicationFieldRef<S, Record<I>, Facts<I>, Scalar<F>, u64, ReadWrite, EqualityPredicate> {
    ApplicationFieldRef::from_schema_types()
}
fn scalar<S, const I: usize, const F: usize>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .field(entity::<S, I>(), field::<S, I, F>())
        .operation_read_field(operation(), field::<S, I, F>())
        .operation_write(operation(), field::<S, I, F>())
}
fn record<S, const I: usize, const FIELDS: usize>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
    reverse: bool,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let mut schema = schema.entity(entity::<S, I>()).aspect(
        entity::<S, I>(),
        ApplicationAspectRef::<S, Record<I>, Facts<I>>::from_schema_identifier("Facts"),
    );
    // Source is preserved. Realization and its output records may be created.
    if I != 0 {
        let relation =
            ApplicationRelationRef::<S, Owner<I>, Record<I>, Record<0>>::from_schema_identifiers(
                RELATION_NAMES[I],
                RECORD_NAMES[I],
                RECORD_NAMES[0],
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
        schema = schema
            .operation_create(operation(), entity::<S, I>())
            .relation(relation, entity::<S, I>(), entity::<S, 0>())
            .operation_read_relation(operation(), relation)
            .operation_link(operation(), relation);
    }
    macro_rules! fields { ($($f:literal),*) => { $(
        if FIELDS > $f { schema = scalar::<S, I, $f>(schema); }
    )* }; }
    if reverse {
        fields!(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    } else {
        fields!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    }
    schema
}
impl<const RECORDS: usize, const FIELDS: usize> ApplicationSchema for Model<RECORDS, FIELDS> {
    const OWNER: &'static str = "worth.query.court.aspect-native";
    const NAME: &'static str = "SourceAndRealization";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration::<RECORDS, FIELDS>(false)
    }
}

pub fn declaration<const RECORDS: usize, const FIELDS: usize>(
    reverse: bool,
) -> Result<ApplicationSchemaDeclaration<Model<RECORDS, FIELDS>>, ApplicationSchemaDeclarationDenial>
{
    declaration_for::<Model<RECORDS, FIELDS>, RECORDS, FIELDS>(reverse, Vec::new())
}

pub(super) fn declaration_for<S: ApplicationSchema, const RECORDS: usize, const FIELDS: usize>(
    reverse: bool,
    paths: Vec<ApplicationAuthorizationPath>,
) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
    assert!((2..=40).contains(&RECORDS));
    assert!((1..=16).contains(&FIELDS));
    let mut schema = ApplicationSchemaDeclarationBuilder::for_schema().operation(
        operation::<S>()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    );
    macro_rules! records { ($($i:literal),*) => { $(
            if RECORDS > $i { schema = record::<S, $i, FIELDS>(schema, reverse); }
        )* }; }
    if reverse {
        records!(
            39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18,
            17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0
        );
    } else {
        records!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
        );
    }
    if !paths.is_empty() {
        schema = authorization::declare(schema, paths);
    }
    schema.build()
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
