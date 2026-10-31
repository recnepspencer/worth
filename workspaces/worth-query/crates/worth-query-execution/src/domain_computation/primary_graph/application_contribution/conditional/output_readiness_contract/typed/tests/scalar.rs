//! Actual installed Scalar field meaning, with unchanged raw Foundational fences.
use super::*;
worth_query_declaration::worth_query_application_schema!(schema Scalar {
    owner: Scalar, version: (1,0), members: |schema| {
        schema.entity(Entity::reference())
            .aspect(Entity::reference(), Identity::reference())
            .field(Entity::reference(), Key::reference())
            .optional_aspect(Entity::reference(), Publication::reference())
            .scalar_aspect_field(Entity::reference(), Value::reference())
    }
});
impl Fixture for Scalar {}
worth_query_declaration::worth_query_field!(Value for S: Fixture, Entity, Publication:
    u64 => schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_declaration::worth_query_field!(Alias for S: Fixture, Entity, Publication:
    u64 => schema::U64ApplicationValueBinding, read_write, no_equality);

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
        FieldExpectation::declared(declaration.erased(), Value::reference::<Scalar>()).unwrap();
    field.seal(&installed).unwrap();
    assert!(matches!(
        field.aspect.shape(),
        AspectShape::Scalar(domain::ScalarAspectType::UInt64)
    ));
    assert_eq!(field.aspect.absence(), AbsenceLaw::Optional);
    assert!(field.mask().is_whole_aspect());
    assert_eq!(
        field.native_binding,
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("Value").unwrap(),
        }
    );
    let raw = domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("Value").unwrap(),
    )]);
    assert_eq!(
        field.aspect.admits_projection_mask(&raw),
        Err(worth_foundational::facade::MaskAdmissibilityDenial::FieldMaskRequiresStruct)
    );
    let alias =
        FieldExpectation::declared(declaration.erased(), Alias::reference::<Scalar>()).unwrap_err();
    assert_eq!(
        alias.field_locus(),
        Some(&schema::ApplicationFieldBindingLocus::new(
            "Entity",
            "Publication",
            "Alias"
        ))
    );
    let contract = WorthQueryOutputReadinessContractBuilder::for_field(
        declaration.erased(),
        Value::reference::<Scalar>(),
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
        Value::reference::<Scalar>(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )
    .unwrap()
    .build_typed::<(), (), ()>()
    .unwrap();
    let node = &contract.definition().semantics().conditional_nodes[0];
    let ordinal = contract
        .delivery_ordinal(&installed, node, Value::reference::<Scalar>())
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
        &[domain::AuthoritativeAspectChangeKind::WholeAspectSet]
    );
    assert!(contract
        .delivery_ordinal(&installed, node, Alias::reference::<Scalar>())
        .is_err());
}

fn scalar_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Entity").unwrap())
        .project(AspectFieldSelector::new("Publication", "Value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("Publication", "Value", "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
