//! Actual installed Scalar field meaning, with unchanged raw Foundational fences.
use crate::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests::*;
struct Scalar;
impl schema::ApplicationSchema for Scalar {
    const OWNER: &'static str = "Scalar";
    const NAME: &'static str = "Scalar";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<4>)?.build()
    }
}
type Value = fragment::FieldSelection<Model<4>, Node<4>, Publication<4>, u64>;

#[test]
fn scalar_readiness_uses_exact_declared_and_installed_binding() {
    let declaration = Scalar::declaration().unwrap();
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new(Scalar::OWNER, 1, 0),
    )
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = domain::WorthQueryInstallationAdmissionProfile::new("readiness", "scalar")
        .admit(package)
        .unwrap();
    let installed = domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration.clone())
    .unwrap();
    let field =
        FieldExpectation::declared_for_test(declaration.erased(), Value::at("value")).unwrap();
    field.seal_for_test(&installed).unwrap();
    assert!(matches!(
        (*field.aspect_for_test()).shape(),
        AspectShape::Struct(_)
    ));
    assert_eq!((*field.aspect_for_test()).absence(), AbsenceLaw::Optional);
    assert!(!field.mask_for_test().is_whole_aspect());
    assert_eq!(
        (*field.native_binding_for_test()),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new(Value::at("value").aspect()).unwrap(),
        }
    );
    let raw = domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("value").unwrap(),
    )]);
    assert_eq!(
        (*field.aspect_for_test()).admits_projection_mask(&raw),
        Ok(())
    );
    let raw_scalar = worth_foundational::facade::AspectContract::scalar(
        domain::AspectKey::new("raw.scalar").unwrap(),
        domain::AspectIdentity(0x0071_93ff),
        domain::AspectContractRevision(1),
        domain::ScalarAspectType::UInt64,
    );
    assert_eq!(
        raw_scalar.admits_projection_mask(&raw),
        Err(worth_foundational::facade::MaskAdmissibilityDenial::FieldMaskRequiresStruct),
    );
    let alias =
        FieldExpectation::declared_for_test(declaration.erased(), Value::at("Alias")).unwrap_err();
    assert_eq!(
        alias.field_locus_for_test(),
        Some(&schema::ApplicationFieldBindingLocus::new(
            "Entity",
            Value::at("value").aspect(),
            "Alias"
        ))
    );
    let contract = WorthQueryOutputReadinessContractBuilder::for_field(
        declaration.erased(),
        Value::at("value"),
        domain::WorthQueryDomainOperationIdentity::new("scalar-readiness", 1),
        "scalar-ready",
        scalar_query(),
        domain::WorthQueryOperationProjectionRole::new("realization").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("scalar-readiness").unwrap(),
        "typed-scalar-readiness-v1",
    )
    .unwrap()
    .field_dependency(
        declaration.erased(),
        Value::at("value"),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )
    .unwrap()
    .build_typed::<(), (), ()>()
    .unwrap();
    let node = &contract.definition().semantics().conditional_nodes[0];
    let ordinal = contract
        .delivery_ordinal(&installed, node, Value::at("value"))
        .unwrap();
    assert_eq!(
        node.dependencies()[ordinal].binding(),
        field.native_binding_for_test()
    );
    assert_eq!(
        node.dependencies()[ordinal].projection_mask(),
        &field.mask_for_test()
    );
    assert_eq!(
        node.dependencies()[ordinal].relevant_changes(),
        &[domain::AuthoritativeAspectChangeKind::FieldSet]
    );
    assert!(contract
        .delivery_ordinal(&installed, node, Value::at("Alias"))
        .is_err());
}

pub fn scalar_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Entity").unwrap())
        .project(AspectFieldSelector::new(Value::at("value").aspect(), "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(Value::at("value").aspect(), "value", "value").unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
