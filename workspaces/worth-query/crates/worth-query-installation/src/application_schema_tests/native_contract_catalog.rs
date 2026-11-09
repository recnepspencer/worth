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
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};

struct CrossOwnerSchema;
impl ApplicationSchema for CrossOwnerSchema {
    const OWNER: &'static str = "native-contract-cross-owner";
    const NAME: &'static str = "CrossOwnerSchema";
    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::from_fragment(super::InstallationFields)?.build()
    }
}
fn identity_aspect() -> &'static str {
    super::FixturePrincipalIdentityField::at("PrincipalIdentityField").aspect()
}
fn identity_id() -> AspectIdentity {
    AspectIdentity(name_identity(
        "installation_fields.TestEntity.IdentityAspect",
    ))
}
fn identity_revision() -> AspectContractRevision {
    TestSchema::declaration().unwrap().erased().members().iter().find_map(|member| match member {
 worth_query_declaration::facade::application_schema::ApplicationSchemaMember::Aspect { revision, .. } => Some(*revision), _ => None,
 }).unwrap()
}

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
    let installed = catalog.aspect("TestEntity", identity_aspect()).unwrap();

    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog.maximum_aspect_identity(), Some(identity_id()));
    assert_eq!(installed.locus().schema(), &first.binding_identity());
    assert_eq!(installed.locus().entity(), "TestEntity");
    assert_eq!(installed.locus().aspect().as_str(), identity_aspect());
    assert_eq!(installed.contract().identity(), identity_id());
    assert_eq!(installed.contract().revision(), identity_revision());
    assert_eq!(installed.contract().key().as_str(), identity_aspect());
    assert_eq!(
        installed.binding(),
        &AspectBinding::EntityField {
            field: FieldKey::new(identity_aspect()).unwrap(),
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
            .aspect("TestEntity", identity_aspect())
            .unwrap()
            .contract(),
        successor_schema
            .native_contracts()
            .aspect("TestEntity", identity_aspect())
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
            .aspect("TestEntity", identity_aspect())
            .unwrap()
            .contract()
            .identity(),
        identity_id()
    );
    assert_ne!(first.binding_identity(), second.binding_identity());
    assert_eq!(index.counters().application_schema_catalogs_compiled, 2);
}

#[test]
fn package_validation_denies_unexportable_native_contracts() {
    use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
    let zero = semantic_collision::fresh(|members| {
        for member in members {
            if let ApplicationSchemaMember::Aspect { revision, .. } = member {
                *revision = AspectContractRevision(0);
            }
        }
    });
    assert_package_contract_denial(
        zero,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractRevisionZero,
    );
    let empty = semantic_collision::fresh(|members| {
        members.retain(|member| !matches!(member, ApplicationSchemaMember::Field { entity, .. } if entity == "FieldEntity"))
    });
    assert_package_contract_denial(
        empty,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractMissingAspectFieldClosure,
    );
}
fn assert_package_contract_denial(
    declaration: worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration,
    expected: WorthQueryPortablePackageValidationDenialKind,
) {
    let owner = declaration.owner().to_owned();
    let denial =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(owner, 1, 0))
            .application_schema_erased(declaration)
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

const fn name_identity(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u64).wrapping_mul(0x100_0000_01b3);
        index += 1;
    }
    hash
}
