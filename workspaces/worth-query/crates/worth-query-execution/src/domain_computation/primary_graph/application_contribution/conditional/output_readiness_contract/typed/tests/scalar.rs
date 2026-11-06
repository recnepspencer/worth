//! Actual installed Scalar field meaning, with unchanged raw Foundational fences.
use super::*;
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
    let field = FieldExpectation::declared(declaration.erased(), Value::at("value")).unwrap();
    field.seal(&installed).unwrap();
    assert!(matches!(field.aspect.shape(), AspectShape::Struct(_)));
    assert_eq!(field.aspect.absence(), AbsenceLaw::Optional);
    assert!(!field.mask().is_whole_aspect());
    assert_eq!(
        field.native_binding,
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new(Value::at("value").aspect()).unwrap(),
        }
    );
    let alias = FieldExpectation::declared(declaration.erased(), Value::at("Alias")).unwrap_err();
    assert_eq!(
        alias.field_locus(),
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
        &field.native_binding
    );
    assert_eq!(
        node.dependencies()[ordinal].projection_mask(),
        &field.mask()
    );
    assert_eq!(
        node.dependencies()[ordinal].relevant_changes(),
        &[domain::AuthoritativeAspectChangeKind::FieldSet]
    );
    assert!(contract
        .delivery_ordinal(&installed, node, Value::at("Alias"))
        .is_err());
}

fn scalar_query() -> CanonicalQueryBundle {
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
