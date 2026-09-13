use super::*;

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
    assert_eq!(
        binding
            .candidate_requirements()
            .resources()
            .maximum_retained_representation_bytes(),
        128
    );
    let envelope = binding
        .operation()
        .contracts()
        .execution_strategy()
        .expect("the installed mutation operation has one execution strategy")
        .envelope();
    assert!(
        envelope.scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems) >= 1,
        "the operation envelope must admit the binding's exact candidate cardinality"
    );
    assert_eq!(
        envelope
            .resource_ceiling(WorthQueryResourceDimension::CandidateRetainedRepresentationBytes),
        128,
        "candidate bytes must derive from the installed binding"
    );
    assert!(
        envelope.scale_ceiling(WorthQuerySemanticScaleAxis::WorkItems) >= 4,
        "validator work must admit the installed binding"
    );
    assert_eq!(
        binding.handler_identity(),
        "worth.query.installation-test.mutation-handler.v1"
    );
    assert_eq!(
        binding.result_identity(),
        TestInputBinding::IDENTITY.as_str()
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
            .installed_mutation_binding::<ChangedCandidateBinding>()
            .err()
            .unwrap()
            .kind(),
        DenialKind::MutationBindingCandidateMeaningChanged
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
