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
schema!(
    LegacySchema,
    LegacyEntity,
    LegacyAspect,
    LegacyField,
    u64,
    U64ApplicationValueBinding,
    0x9174_0012
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
fn catalogue_identity_retains_clones_but_declines_independent_installations() {
    let declaration = NestedSchema::declaration().unwrap();
    let index = index::<NestedSchema>();
    let first = index.bind_application_schema(declaration.clone()).unwrap();
    let retained = first.value_bindings().clone();
    assert!(first.value_bindings().same_installation_as(&retained));
    let rebound = index.bind_application_schema(declaration.clone()).unwrap();
    let independent = self::index::<NestedSchema>()
        .bind_application_schema(declaration)
        .unwrap();
    for other in [&rebound, &independent] {
        assert_eq!(
            first.value_meaning().unwrap(),
            other.value_meaning().unwrap()
        );
        assert!(!retained.same_installation_as(other.value_bindings()));
    }
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
    let declared = collect_declared_fields(declaration.erased()).unwrap();
    let contract = declared[recipe.locus()];
    validate_contract(recipe.locus(), contract, recipe, schema.native_contracts()).unwrap();
    let changed_shape = ApplicationCanonicalValueContract::of::<String>(BOUNDS);
    let changed_bounds = ApplicationCanonicalValueContract::of::<Vec<String>>(
        ValueDecodeBounds::new(1024, 64, 8, 8),
    );
    for canonical in [None, Some(&changed_shape), Some(&changed_bounds)] {
        let changed = (
            contract.0, contract.1, contract.2, contract.3, canonical, contract.5, contract.6,
        );
        let denial = validate_contract(recipe.locus(), changed, recipe, schema.native_contracts())
            .unwrap_err();
        assert_eq!(denial.kind(), DenialKind::ContractMismatch);
    }
}

#[test]
fn installed_value_meaning_is_consumed_from_exact_native_and_executable_catalogues() {
    use worth_foundational::facade::AspectShape;
    let schema = index::<NestedSchema>()
        .bind_application_schema(NestedSchema::declaration().unwrap())
        .unwrap();
    let meanings = schema.value_meaning().unwrap();
    assert_eq!(meanings.len(), 1);
    let meaning = &meanings[0];
    let native = schema
        .native_contracts()
        .aspect("Entity", "Aspect")
        .unwrap();
    assert_eq!(meaning.identity(), native.contract().identity());
    assert_eq!(meaning.revision(), native.contract().revision());
    assert_eq!(meaning.fields().len(), 1);
    let AspectShape::Struct(shape) = native.contract().shape() else {
        panic!("declared field has native struct closure")
    };
    assert_eq!(
        shape.fields()[0].codec(),
        meaning.fields()[0].codec_digest()
    );
    assert!(
        shape.fields()[0].codec().is_some(),
        "canonical meaning cannot disappear behind the String carrier"
    );
    let ApplicationSchemaMember::Field { canonical, .. } = meaning.fields()[0].member() else {
        unreachable!()
    };
    assert_eq!(
        canonical.as_ref(),
        schema
            .value_bindings()
            .field("Entity", "Aspect", "Field")
            .unwrap()
            .canonical_contract()
    );
}

#[test]
fn installed_codec_admission_refuses_missing_wrong_and_unexpected_native_meaning() {
    let declaration = NestedSchema::declaration().unwrap();
    let schema = index::<NestedSchema>()
        .bind_application_schema(declaration.clone())
        .unwrap();
    let field = declaration
        .erased()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::Field { .. }))
        .unwrap();
    let expected = ApplicationFieldValueMeaning::from_member(field)
        .unwrap()
        .unwrap()
        .codec_digest()
        .unwrap();
    let mut changed = field.clone();
    let ApplicationSchemaMember::Field { canonical, .. } = &mut changed else {
        unreachable!()
    };
    *canonical = Some(ApplicationCanonicalValueContract::of::<String>(BOUNDS));
    let wrong = ApplicationFieldValueMeaning::from_member(&changed)
        .unwrap()
        .unwrap()
        .codec_digest()
        .unwrap();
    assert_ne!(
        expected, wrong,
        "the changed grammar uses the same String carrier"
    );
    for codec in [Some(expected), None, Some(wrong)] {
        let native = imported_native_with_codec(&schema, codec);
        let result = compile_value_binding_catalog(
            &schema.binding_identity(),
            declaration.erased(),
            declaration.member_provenance(),
            &native,
        );
        if codec == Some(expected) {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().kind(), DenialKind::ContractMismatch);
        }
    }
    let legacy_declaration = LegacySchema::declaration().unwrap();
    let legacy = index::<LegacySchema>()
        .bind_application_schema(legacy_declaration.clone())
        .unwrap();
    for codec in [None, Some(expected)] {
        let native = imported_native_with_codec(&legacy, codec);
        let result = compile_value_binding_catalog(
            &legacy.binding_identity(),
            legacy_declaration.erased(),
            legacy_declaration.member_provenance(),
            &native,
        );
        if codec.is_none() {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().kind(), DenialKind::ContractMismatch);
        }
    }
}

fn imported_native_with_codec<Schema: ApplicationSchema>(
    schema: &crate::application_schema::WorthQueryInstalledApplicationSchema<Schema>,
    codec: Option<worth_foundational::facade::CanonicalDigestId>,
) -> crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog {
    use crate::application_schema::{
        compile_native_contract_catalog, compile_portable_native_contract_records,
    };
    use crate::package::{
        WorthQueryPortableNativeAspectContractParts, WorthQueryPortableNativeAspectContractRecord,
    };
    use worth_foundational::facade::{aspects, AspectShape, FieldDeclaration, StructAspectShape};
    let record = compile_portable_native_contract_records(schema.installed_declaration())
        .unwrap()
        .remove(0);
    let AspectShape::Struct(shape) = record.contract().shape() else {
        unreachable!()
    };
    assert_eq!(
        shape.fields().len(),
        1,
        "the fixture changes only one real native field"
    );
    let source = &shape.fields()[0];
    let field = FieldDeclaration::new(
        source.key().clone(),
        source.value_type(),
        source.requirement(),
        source.absence(),
        source.evolution(),
    )
    .unwrap();
    let field = match codec {
        Some(codec) => field.with_codec(codec),
        None => field,
    };
    let contract = aspects()
        .contract()
        .for_key(record.aspect().clone())
        .identified_by(record.contract().identity())
        .at_revision(record.contract().revision())
        .struct_aspect(StructAspectShape::new([field]).unwrap());
    // The imported record is deliberately descriptive/untrusted; the normal
    // native compiler and executable-binding admission must agree on its codec.
    let imported = WorthQueryPortableNativeAspectContractRecord::from_untrusted_parts(
        WorthQueryPortableNativeAspectContractParts {
            schema: record.schema().into(),
            entity: record.entity().into(),
            aspect: record.aspect().clone(),
            contract,
            fields: record.fields().cloned().collect(),
            binding: record.binding().clone(),
        },
    );
    compile_native_contract_catalog(
        &schema.binding_identity(),
        &[imported],
        schema.installation_canonical_work(),
    )
    .unwrap()
}
