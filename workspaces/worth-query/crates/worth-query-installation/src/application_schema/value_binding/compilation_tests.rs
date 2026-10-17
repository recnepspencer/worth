use super::*;
use worth_query_declaration::facade::application_schema::*;

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchemaDenialKind,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 128, 8, 8);
worth_query_declaration::worth_query_value_binding! {
    NestedBinding for Vec<String> {
        identity: "worth.tests.admitted-catalogue.nested",
        scalar: String,
        canonical: BOUNDS,
    }
}

macro_rules! schema {
    ($Schema:ident, $Entity:ident, $Aspect:ident, $Field:ident, $Value:ty, $Binding:ty, $id:expr) => {
        worth_query_declaration::worth_query_application_schema! {
            schema $Schema {
                owner: $Schema, version: (1, 0),
                members: |schema| {
                    schema.entity($Entity::reference())
                        .aspect($Entity::reference(), $Aspect::reference())
                        .field($Entity::reference(), $Field::reference())
                }
            }
        }
        worth_query_declaration::worth_query_entity!($Entity for $Schema);
        worth_query_declaration::worth_query_aspect!(
            $Aspect for $Schema, $Entity;
            identity = AspectIdentity($id),
            revision = AspectContractRevision(1),
        );
        worth_query_declaration::worth_query_field!(
            $Field for $Schema, $Entity, $Aspect: $Value => $Binding,
            read_only, no_equality
        );
    };
}
schema!(
    NestedSchema,
    Entity,
    Aspect,
    Field,
    Vec<String>,
    NestedBinding,
    0x9174_0010
);
schema!(
    OpaqueSchema,
    OpaqueEntity,
    OpaqueAspect,
    OpaqueField,
    String,
    OpaqueBinding,
    0x9174_0011
);

struct OpaqueBinding;
impl ApplicationScalarValueBinding for OpaqueBinding {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.tests.admitted-catalogue.opaque";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        worth_foundational::facade::ScalarAspectType::String;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::of::<String>(BOUNDS))
    }
    fn validate(_: &String) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(
        _: &String,
    ) -> Result<worth_foundational::facade::AspectValue, ApplicationValueEncodeDenial> {
        unreachable!("installation must refuse the missing admitted contract")
    }
}
impl ApplicationReadableScalarValueBinding for OpaqueBinding {
    fn decode(
        _: &worth_foundational::facade::AspectValue,
    ) -> Result<String, ApplicationValueDecodeDenial> {
        panic!("missing admitted support cannot fall back to the ordinary decoder")
    }
}

fn index<Schema: ApplicationSchema>() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        1,
        0,
    ))
    .application_schema(Schema::declaration().unwrap())
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

#[test]
fn canonical_admission_dispatch_is_the_exact_installed_binding_and_shared_catalogue() {
    let schema = index::<NestedSchema>()
        .bind_application_schema(NestedSchema::declaration().unwrap())
        .unwrap();
    let catalogue = schema.value_bindings();
    let retained = catalogue.clone();
    let binding = catalogue.field("Entity", "Aspect", "Field").unwrap();
    assert!(std::ptr::eq(
        binding,
        retained.field("Entity", "Aspect", "Field").unwrap()
    ));
    assert!(binding.matches_binding::<NestedBinding>());
    assert!(!binding.matches_binding::<OpaqueBinding>());
    assert!(binding.supports_admitted_decode());
    let raw = NestedBinding::encode(&vec!["actual nested value".into()]).unwrap();
    let mut grants = 0;
    let value = binding
        .decode_admitted::<Vec<String>>(&raw, &mut |_, _| {
            grants += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(value, vec!["actual nested value"]);
    assert!(grants > 1);
    let mut denied = 0;
    assert!(binding
        .decode_admitted::<Vec<String>>(&raw, &mut |_, _| {
            denied += 1;
            Err(ValueCodecDenial::Admission)
        })
        .is_err());
    assert_eq!(denied, 1);
    assert!(binding
        .decode_admitted::<u64>(&raw, &mut |_, _| panic!(
            "wrong output type must fail before admission"
        ))
        .is_err());
}

#[test]
fn canonical_admission_installation_refuses_missing_owner_support() {
    let denial = index::<OpaqueSchema>()
        .bind_application_schema(OpaqueSchema::declaration().unwrap())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingContractMismatch
    );
}

#[test]
fn canonical_admission_compilation_rejects_changed_shape_bounds_and_absent_meaning() {
    let declaration = NestedSchema::declaration().unwrap();
    let schema = index::<NestedSchema>()
        .bind_application_schema(declaration.clone())
        .unwrap();
    let recipe = &declaration.member_provenance().field_bindings()[0];
    let declared = collect_declared_fields(declaration.erased());
    let contract = declared[recipe.locus()];
    validate_contract(recipe.locus(), contract, recipe, schema.native_contracts()).unwrap();
    let changed_shape = ApplicationCanonicalValueContract::of::<String>(BOUNDS);
    let changed_bounds = ApplicationCanonicalValueContract::of::<Vec<String>>(
        ValueDecodeBounds::new(1024, 64, 8, 8),
    );
    for canonical in [None, Some(&changed_shape), Some(&changed_bounds)] {
        let changed = (contract.0, contract.1, contract.2, contract.3, canonical);
        let denial = validate_contract(recipe.locus(), changed, recipe, schema.native_contracts())
            .unwrap_err();
        assert_eq!(denial.kind(), DenialKind::ContractMismatch);
    }
}
