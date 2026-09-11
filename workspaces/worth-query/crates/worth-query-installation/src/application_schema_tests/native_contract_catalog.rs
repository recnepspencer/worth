use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectShape, FieldKey, FieldRequirement,
    ScalarAspectType,
};

use super::{installed_index, TestSchema};

mod exact_contract;
mod semantic_collision;
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageValidationDenialKind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder,
};

worth_query_declaration::worth_query_entity!(ZeroEntity for ZeroRevisionSchema);
worth_query_declaration::worth_query_aspect!(
    ZeroAspect for ZeroRevisionSchema, ZeroEntity;
    identity = AspectIdentity(1),
    revision = AspectContractRevision(0),
);
worth_query_declaration::worth_query_entity!(DuplicateEntity for DuplicateIdentitySchema);
worth_query_declaration::worth_query_aspect!(
    DuplicateAspectA for DuplicateIdentitySchema, DuplicateEntity;
    identity = AspectIdentity(7),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_aspect!(
    DuplicateAspectB for DuplicateIdentitySchema, DuplicateEntity;
    identity = AspectIdentity(7),
    revision = AspectContractRevision(2),
);
worth_query_declaration::worth_query_entity!(EmptyEntity for EmptyAspectSchema);
worth_query_declaration::worth_query_aspect!(
    EmptyAspect for EmptyAspectSchema, EmptyEntity;
    identity = AspectIdentity(9),
    revision = AspectContractRevision(1),
);

struct ZeroRevisionSchema;
struct DuplicateIdentitySchema;
struct EmptyAspectSchema;
struct CrossOwnerSchema;

worth_query_declaration::worth_query_entity!(CrossEntity for CrossOwnerSchema);
worth_query_declaration::worth_query_aspect!(
    CrossAspect for CrossOwnerSchema, CrossEntity;
    identity = AspectIdentity(0x9161200c),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    CrossField for CrossOwnerSchema, CrossEntity, CrossAspect:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);

impl ApplicationSchema for CrossOwnerSchema {
    const OWNER: &'static str = "native-contract-cross-owner";
    const NAME: &'static str = "CrossOwnerSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(CrossEntity::reference())
            .aspect(CrossEntity::reference(), CrossAspect::reference())
            .field(CrossEntity::reference(), CrossField::reference())
            .build()
    }
}

macro_rules! schema_identity {
    ($schema:ty, $name:literal) => {
        impl ApplicationSchema for $schema {
            const OWNER: &'static str = "native-contract-denial";
            const NAME: &'static str = $name;
            const MAJOR: u32 = 1;
            const MINOR: u32 = 0;

            fn declaration() -> Result<
                ApplicationSchemaDeclaration<Self>,
                worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                unreachable!("each denial fixture authors its declaration explicitly")
            }
        }
    };
}

schema_identity!(ZeroRevisionSchema, "ZeroRevisionSchema");
schema_identity!(DuplicateIdentitySchema, "DuplicateIdentitySchema");
schema_identity!(EmptyAspectSchema, "EmptyAspectSchema");

#[test]
fn installed_catalog_retains_the_exact_declared_native_contract_once() {
    let index = installed_index();
    let counters = index.counters();
    let first = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let _ = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let catalog = first.native_contracts();
    let installed = catalog.aspect("TestEntity", "IdentityAspect").unwrap();

    assert_eq!(catalog.len(), 1);
    assert_eq!(
        catalog.maximum_aspect_identity(),
        Some(AspectIdentity(0x9161200c))
    );
    assert_eq!(installed.locus().schema(), &first.binding_identity());
    assert_eq!(installed.locus().entity(), "TestEntity");
    assert_eq!(installed.locus().aspect().as_str(), "IdentityAspect");
    assert_eq!(installed.contract().identity(), AspectIdentity(0x9161200c));
    assert_eq!(installed.contract().revision(), AspectContractRevision(1));
    assert_eq!(installed.contract().key().as_str(), "IdentityAspect");
    assert_eq!(
        installed.binding(),
        &AspectBinding::EntityField {
            field: FieldKey::new("IdentityAspect").unwrap(),
        }
    );
    let AspectShape::Struct(shape) = installed.contract().shape() else {
        panic!("application aspects must compile to exact native struct contracts")
    };
    let fields = shape.fields();
    assert_eq!(fields.len(), 3);
    assert!(fields.iter().any(|field| {
        field.key().as_str() == "PrincipalIdentityField"
            && field.value_type() == ScalarAspectType::UInt64
            && field.requirement() == FieldRequirement::Required
    }));
    assert!(!installed.canonical_contract_material().is_empty());
    assert_eq!(catalog.counters().catalogs_compiled(), 1);
    assert_eq!(catalog.counters().contracts_compiled(), 1);
    assert_eq!(catalog.counters().fields_compiled(), 3);
    assert_eq!(catalog.counters().canonical_contract_bases_prepared(), 1);
    assert_eq!(counters.application_schema_catalogs_compiled, 1);
    assert_eq!(counters.application_aspect_contracts_compiled, 1);
    assert_eq!(counters.application_aspect_fields_compiled, 3);
    assert_eq!(counters.application_aspect_canonical_bases_prepared, 1);
    assert_eq!(
        index.counters(),
        counters,
        "typed binding must not compile again"
    );
}

#[test]
fn rebuild_and_successor_each_compile_one_generation_affine_catalog() {
    let current = installed_index();
    let rebuilt = current.rebuild();
    let successor = current.successor_generation();

    for index in [&current, &rebuilt, &successor] {
        let counters = index.counters();
        assert_eq!(counters.application_schema_catalogs_compiled, 1);
        assert_eq!(counters.application_aspect_contracts_compiled, 1);
        assert_eq!(counters.application_aspect_fields_compiled, 3);
        assert_eq!(counters.application_aspect_canonical_bases_prepared, 1);
    }

    let rebuilt_first = rebuilt
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let _ = rebuilt
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let successor_schema = successor
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    assert_ne!(
        rebuilt_first.binding_identity(),
        successor_schema.binding_identity()
    );
    assert_eq!(
        rebuilt_first
            .native_contracts()
            .aspect("TestEntity", "IdentityAspect")
            .unwrap()
            .contract(),
        successor_schema
            .native_contracts()
            .aspect("TestEntity", "IdentityAspect")
            .unwrap()
            .contract()
    );
}

#[test]
fn aspect_identity_namespace_is_local_to_one_installed_schema_binding() {
    let packages = [
        admitted_package(TestSchema::declaration().unwrap()),
        admitted_package(CrossOwnerSchema::declaration().unwrap()),
    ];
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        packages,
    )
    .unwrap();
    let first = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let second = index
        .bind_application_schema(CrossOwnerSchema::declaration().unwrap())
        .unwrap();
    assert_eq!(
        first.native_contracts().maximum_aspect_identity(),
        second.native_contracts().maximum_aspect_identity()
    );
    assert_eq!(
        second
            .native_contracts()
            .aspect("CrossEntity", "CrossAspect")
            .unwrap()
            .contract()
            .identity(),
        AspectIdentity(0x9161200c)
    );
    assert_ne!(first.binding_identity(), second.binding_identity());
    assert_eq!(index.counters().application_schema_catalogs_compiled, 2);
}

#[test]
fn package_validation_denies_unexportable_native_contracts() {
    let zero_entity =
        ApplicationEntityRef::<ZeroRevisionSchema, ZeroEntity>::from_schema_identifier(
            "ZeroEntity",
        );
    let zero = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(zero_entity)
        .aspect(
            zero_entity,
            worth_query_declaration::facade::application_schema::ApplicationAspectRef::<
                ZeroRevisionSchema,
                ZeroEntity,
                ZeroAspect,
            >::from_schema_identifier("ZeroAspect"),
        )
        .build()
        .unwrap();
    assert_package_contract_denial(
        zero,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractRevisionZero,
    );

    let duplicate_entity =
        ApplicationEntityRef::<DuplicateIdentitySchema, DuplicateEntity>::from_schema_identifier(
            "DuplicateEntity",
        );
    let duplicate = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(duplicate_entity)
        .aspect(
            duplicate_entity,
            worth_query_declaration::facade::application_schema::ApplicationAspectRef::<
                DuplicateIdentitySchema,
                DuplicateEntity,
                DuplicateAspectA,
            >::from_schema_identifier("DuplicateAspectA"),
        )
        .aspect(
            duplicate_entity,
            worth_query_declaration::facade::application_schema::ApplicationAspectRef::<
                DuplicateIdentitySchema,
                DuplicateEntity,
                DuplicateAspectB,
            >::from_schema_identifier("DuplicateAspectB"),
        )
        .build()
        .unwrap();
    assert_package_contract_denial(
        duplicate,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateAspectIdentity,
    );

    let empty_entity =
        ApplicationEntityRef::<EmptyAspectSchema, EmptyEntity>::from_schema_identifier(
            "EmptyEntity",
        );
    let empty = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(empty_entity)
        .aspect(
            empty_entity,
            worth_query_declaration::facade::application_schema::ApplicationAspectRef::<
                EmptyAspectSchema,
                EmptyEntity,
                EmptyAspect,
            >::from_schema_identifier("EmptyAspect"),
        )
        .build()
        .unwrap();
    assert_package_contract_denial(
        empty,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractMissingAspectFieldClosure,
    );
}

fn assert_package_contract_denial<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    expected: WorthQueryPortablePackageValidationDenialKind,
) where
    Schema: ApplicationSchema,
{
    let denial = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration)
    .validate()
    .unwrap_err();
    assert_eq!(denial.kind(), expected);
}

fn admitted_package<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
) -> crate::facade::WorthQueryAdmittedPortableDomainPackage
where
    Schema: ApplicationSchema,
{
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration)
    .validate()
    .unwrap();
    WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap()
}
