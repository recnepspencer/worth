use super::*;

#[test]
fn retained_mutation_binding_checks_current_issuer() {
    use crate::facade::WorthQueryApplicationOperationInstallationDenialKind as DenialKind;

    let index = installed_mutation_index();
    let schema = index
        .bind_application_schema(MutationSchema::declaration().unwrap())
        .unwrap();
    let binding = schema
        .installed_mutation_binding::<MutationBinding>()
        .unwrap();
    let current = index
        .validate_retained_mutation_binding(&schema, &binding)
        .unwrap();
    assert_eq!(
        current.operation().operation(),
        binding.operation().operation()
    );
    assert_eq!(current.principal_binding().binding(), "PrincipalBinding");

    let rebuilt = index.rebuild();
    assert!(rebuilt
        .validate_retained_mutation_binding(&schema, &binding)
        .is_ok());
    let successor = index.successor_generation();
    assert!(matches!(
        successor.validate_retained_mutation_binding(&schema, &binding),
        Err(denial) if denial.kind() == DenialKind::StaleGeneration
    ));
    let foreign = installed_mutation_index();
    assert!(matches!(
        foreign.validate_retained_mutation_binding(&schema, &binding),
        Err(denial) if denial.kind() == DenialKind::ForeignRuntime
    ));
}

#[test]
fn installed_mutation_binding_resolves_exact_callback_free_contracts() {
    let schema = installed_mutation_index()
        .bind_application_schema(MutationSchema::declaration().unwrap())
        .unwrap();
    let binding = schema
        .installed_mutation_binding::<MutationBinding>()
        .unwrap();

    assert_eq!(binding.operation().operation(), "TestOperation");
    assert_eq!(binding.principal_binding().binding(), "PrincipalBinding");
    assert_eq!(
        binding.scope_contract().field().locus().field(),
        "PrincipalIdentityField"
    );
    let envelope = binding
        .operation()
        .contracts()
        .execution_strategy()
        .expect("the installed mutation operation has one execution strategy")
        .envelope();
    assert_eq!(
        envelope.boundary(),
        worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary::Atomic
    );
    assert_eq!(
        envelope.optional_scale_ceiling(WorthQuerySemanticScaleAxis::ModelSize),
        None
    );
    assert_eq!(
        envelope.optional_resource_ceiling(WorthQueryResourceDimension::QueueDepth),
        None
    );
}

#[test]
fn typed_mutation_lookup_denies_each_changed_contract_axis() {
    let schema = installed_mutation_index()
        .bind_application_schema(MutationSchema::declaration().unwrap())
        .unwrap();
    use crate::facade::WorthQueryApplicationOperationInstallationDenialKind as DenialKind;

    assert_eq!(
        schema
            .installed_mutation_binding::<MissingMutationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingNotInstalled
    );

    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedOperationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingOperationMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedPrincipalMutationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingPrincipalMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedScopeBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingScopeMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedHandlerBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingHandlerMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedDenialMutationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingHandlerMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedResultMutationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingResultMeaningChanged
    );
    assert_eq!(
        schema
            .installed_mutation_binding::<ChangedOutputMutationBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingOutputMeaningChanged
    );
}

fn installed_mutation_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        MutationSchema::OWNER,
        MutationSchema::MAJOR,
        MutationSchema::MINOR,
    ))
    .application_schema(MutationSchema::declaration().unwrap())
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
