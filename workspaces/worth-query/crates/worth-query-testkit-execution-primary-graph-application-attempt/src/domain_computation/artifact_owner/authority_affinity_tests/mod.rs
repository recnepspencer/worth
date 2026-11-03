use std::sync::Arc;
use worth_foundational::facade::CanonicalizationRuleVersion;
use worth_foundational::facade::FoundationalPerformanceCounterName;
use worth_foundational::facade::RetentionDeliveryProfile;
use worth_query_execution::court_doors_for_test::domain_computation::artifact_owner::production_authority::WorthQueryArtifactProductionAuthorityParts;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionGeneration;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryInstalledDomainExecutionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistry;
use worth_query_installation::facade::WorthQueryArtifactClassification;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityContract;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityWindow;
use worth_query_installation::facade::WorthQueryArtifactContentIdentityContract;
use worth_query_installation::facade::WorthQueryArtifactDeletionPosture;
use worth_query_installation::facade::WorthQueryArtifactDeterminismPosture;
use worth_query_installation::facade::WorthQueryArtifactDowngradePosture;
use worth_query_installation::facade::WorthQueryArtifactEvidenceContract;
use worth_query_installation::facade::WorthQueryArtifactFamily;
use worth_query_installation::facade::WorthQueryArtifactLegalHoldPosture;
use worth_query_installation::facade::WorthQueryArtifactLifecycleContract;
use worth_query_installation::facade::WorthQueryArtifactOccurrenceContract;
use worth_query_installation::facade::WorthQueryArtifactOwnershipContract;
use worth_query_installation::facade::WorthQueryArtifactProtocolVersion;
use worth_query_installation::facade::WorthQueryArtifactRedactionPosture;
use worth_query_installation::facade::WorthQueryArtifactReproducibilityClass;
use worth_query_installation::facade::WorthQueryArtifactReproducibilityContract;
use worth_query_installation::facade::WorthQueryArtifactRetirementRule;
use worth_query_installation::facade::WorthQueryArtifactSchemaVersion;
use worth_query_installation::facade::WorthQueryArtifactVersionSupport;
use worth_query_installation::facade::WorthQueryCandidateSearchContract;
use worth_query_installation::facade::WorthQueryConvergenceContract;
use worth_query_installation::facade::WorthQueryDecisionRecordContract;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity;
use worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority;
use worth_query_installation::facade::WorthQueryInstalledPackageIndex;
use worth_query_installation::facade::WorthQueryPortableArtifactContract;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;
use worth_query_installation::facade::WorthQueryStructuralCounterContract;
use worth_query_installation::facade::WorthQueryTransformationEvidenceContract;
pub struct PrimaryFamily;
pub struct AlternateFamily;
impl WorthQueryArtifactFamily for PrimaryFamily {
    const SEMANTIC_FAMILY: &'static str = "WORTH.tests.affinity.primary";
}
impl WorthQueryArtifactFamily for AlternateFamily {
    const SEMANTIC_FAMILY: &'static str = "WORTH.tests.affinity.alternate";
}
pub fn assert_standard_mismatch(
    contracts: &InstalledContracts,
    expected_kind: WorthQueryArtifactDenialKind,
    mutate: impl FnOnce(&mut WorthQueryArtifactProductionAuthorityParts),
) {
    let runtime = runtime();
    let domain = domain_authority(&runtime);
    let expected = authority(Arc::clone(&contracts.expected), Arc::clone(&domain));
    assert_exact_authority_passes(&expected);
    let mut candidate = authority_parts(Arc::clone(&contracts.expected), domain);
    mutate(&mut candidate);
    assert_candidate_denied(expected, candidate, expected_kind);
}
pub fn assert_contract_mismatch(
    contracts: &InstalledContracts,
    candidate_contract: Arc<WorthQueryInstalledArtifactContractAuthority>,
    expected_kind: WorthQueryArtifactDenialKind,
) {
    let runtime = runtime();
    let domain = domain_authority(&runtime);
    let expected = authority(Arc::clone(&contracts.expected), Arc::clone(&domain));
    assert_exact_authority_passes(&expected);
    assert_candidate_denied(
        expected,
        authority_parts(candidate_contract, domain),
        expected_kind,
    );
}
pub fn assert_runtime_mismatch(contracts: &InstalledContracts) {
    let expected_runtime = runtime();
    let foreign_runtime = runtime();
    let expected = authority(
        Arc::clone(&contracts.expected),
        domain_authority(&expected_runtime),
    );
    assert_exact_authority_passes(&expected);
    assert_candidate_denied(
        expected,
        authority_parts(
            Arc::clone(&contracts.expected),
            domain_authority(&foreign_runtime),
        ),
        WorthQueryArtifactDenialKind::ForeignRuntime,
    );
}
pub fn assert_generation_mismatch(contracts: &InstalledContracts) {
    let mut runtime = runtime();
    let expected_domain = domain_authority(&runtime);
    let expected = authority(Arc::clone(&contracts.expected), expected_domain);
    assert_exact_authority_passes(&expected);
    let successor = Arc::new(runtime.installed_packages().successor_generation());
    runtime.commit_successor_installation(successor).unwrap();
    let current_successor = domain_authority(&runtime);
    assert_candidate_denied(
        expected,
        authority_parts(Arc::clone(&contracts.expected), current_successor),
        WorthQueryArtifactDenialKind::StaleInstallationGeneration,
    );
}
pub fn assert_exact_authority_passes(expected: &Arc<WorthQueryArtifactProductionAuthority>) {
    let admission = WorthQueryArtifactProductionAuthority::admit(
        expected,
        WorthQueryArtifactProductionEvidence::new("provenance", "dependency"),
    );
    WorthQueryArtifactProductionAuthority::validate_admission(expected, &admission).unwrap();
}
pub fn assert_candidate_denied(
    expected: Arc<WorthQueryArtifactProductionAuthority>,
    candidate: WorthQueryArtifactProductionAuthorityParts,
    expected_kind: WorthQueryArtifactDenialKind,
) {
    let candidate = WorthQueryArtifactProductionAuthority::mint_for_test(candidate);
    let admission = WorthQueryArtifactProductionAuthority::admit(
        &candidate,
        WorthQueryArtifactProductionEvidence::new("provenance", "dependency"),
    );
    let denial = WorthQueryArtifactProductionAuthority::validate_admission(&expected, &admission)
        .unwrap_err();
    assert_eq!(denial.kind(), expected_kind);
}
pub fn authority(
    contract: Arc<WorthQueryInstalledArtifactContractAuthority>,
    domain_authority: Arc<WorthQueryInstalledDomainExecutionAuthority>,
) -> Arc<WorthQueryArtifactProductionAuthority> {
    WorthQueryArtifactProductionAuthority::mint_for_test(authority_parts(contract, domain_authority))
}
pub fn authority_parts(
    contract: Arc<WorthQueryInstalledArtifactContractAuthority>,
    domain_authority: Arc<WorthQueryInstalledDomainExecutionAuthority>,
) -> WorthQueryArtifactProductionAuthorityParts {
    WorthQueryArtifactProductionAuthorityParts {
        contract,
        domain_authority,
        operation_identity: "installed-operation".into(),
        binding_identity: "installed-binding".into(),
        run_identity: "installed-run".into(),
        stage_identity: "producer".into(),
        basis_identity: "installed-basis".into(),
        registry: Arc::new(WorthQueryWorkflowArtifactRegistry::new_for_test(
            "installed-run".into(),
        )),
        production_generation: WorthQueryArtifactProductionGeneration::initial_for_test(),
    }
}
pub fn runtime() -> WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0
}
pub fn domain_authority(
    runtime: &WorthQueryExecutionRuntime,
) -> Arc<WorthQueryInstalledDomainExecutionAuthority> {
    WorthQueryInstalledDomainExecutionAuthority::mint_for_test(
        runtime.authority_identity(),
        "WORTH.tests.affinity.owner",
        runtime.installed_packages().generation(),
        runtime.retain_current_generation_for_test(),
    )
}
pub struct InstalledContracts {
    pub expected: Arc<WorthQueryInstalledArtifactContractAuthority>,
    pub foreign_owner: Arc<WorthQueryInstalledArtifactContractAuthority>,
    pub foreign_family: Arc<WorthQueryInstalledArtifactContractAuthority>,
    pub foreign_version: Arc<WorthQueryInstalledArtifactContractAuthority>,
}
pub fn installed_artifact_contract_for_managed_run(
) -> Arc<WorthQueryInstalledArtifactContractAuthority> {
    installed_contracts().expected
}
pub fn installed_contracts() -> InstalledContracts {
    let owner = "WORTH.tests.affinity.owner";
    let foreign_owner = "WORTH.tests.affinity.foreign-owner";
    let primary_v1 = contract::<PrimaryFamily>(owner, 1);
    let primary_v2 = contract::<PrimaryFamily>(owner, 2);
    let alternate_v1 = contract::<AlternateFamily>(owner, 1);
    let foreign_owner_v1 = contract::<PrimaryFamily>(foreign_owner, 1);
    let owner_package = admitted_package(owner, [primary_v1, primary_v2, alternate_v1], true);
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [owner_package],
    )
    .unwrap();
    let foreign_index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted_package(foreign_owner, [foreign_owner_v1], false)],
    )
    .unwrap();
    let installed = |index: &WorthQueryInstalledPackageIndex, owner, family, version| {
        Arc::new(
            index
                .artifact_contract(
                    owner,
                    family,
                    WorthQueryArtifactSchemaVersion::new(version),
                    WorthQueryArtifactProtocolVersion::new(1),
                )
                .unwrap(),
        )
    };
    InstalledContracts {
        expected: installed(&index, owner, PrimaryFamily::SEMANTIC_FAMILY, 1),
        foreign_owner: installed(
            &foreign_index,
            foreign_owner,
            PrimaryFamily::SEMANTIC_FAMILY,
            1,
        ),
        foreign_family: installed(&index, owner, AlternateFamily::SEMANTIC_FAMILY, 1),
        foreign_version: installed(&index, owner, PrimaryFamily::SEMANTIC_FAMILY, 2),
    }
}
pub fn admitted_package<const N: usize>(
    owner: &str,
    contracts: [WorthQueryPortableArtifactContract; N],
    include_alternate: bool,
) -> worth_query_installation::facade::WorthQueryAdmittedPortableDomainPackage {
    let package = contracts.into_iter().fold(
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(owner, 1, 0)),
        WorthQueryPortableDomainPackage::artifact_contract,
    );
    let mut profile = WorthQueryInstallationAdmissionProfile::new("support-v1", "config-v1")
        .artifact_version::<PrimaryFamily>(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactVersionSupport::Admitted,
        )
        .artifact_version::<PrimaryFamily>(
            WorthQueryArtifactSchemaVersion::new(2),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactVersionSupport::Admitted,
        );
    if include_alternate {
        profile = profile.artifact_version::<AlternateFamily>(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactVersionSupport::Admitted,
        );
    }
    profile.admit(package.validate().unwrap()).unwrap()
}
pub fn contract<F: WorthQueryArtifactFamily>(
    owner: &str,
    schema_version: u32,
) -> WorthQueryPortableArtifactContract {
    WorthQueryPortableArtifactContract::declare::<F>(
        WorthQueryArtifactSchemaVersion::new(schema_version),
        WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "WORTH.tests.affinity.projection",
            CanonicalizationRuleVersion::new("affinity-v1").unwrap(),
        ),
    )
    .ownership(WorthQueryArtifactOwnershipContract::domain_payload(
        owner,
        "WORTH.tests.affinity.provider",
    ))
    .occurrence(WorthQueryArtifactOccurrenceContract::independent_per_execution())
    .evidence(WorthQueryArtifactEvidenceContract::new(
        "basis",
        "provenance",
        "dependency",
        "invalidation",
        "equivalence",
    ))
    .reproducibility(WorthQueryArtifactReproducibilityContract::new(
        WorthQueryArtifactReproducibilityClass::ExactDeterministic,
        WorthQueryArtifactDeterminismPosture::Deterministic,
        worth_query_installation::facade::WorthQueryArtifactComparisonAuthority::ExactCanonicalValue,
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    ))
    .search(WorthQueryCandidateSearchContract::not_applicable())
    .convergence(WorthQueryConvergenceContract::NotIterative)
    .transformation(WorthQueryTransformationEvidenceContract::not_a_transformation())
    .access_path(worth_query_installation::facade::WorthQueryArtifactAccessPathContract::denied())
    .carriage(worth_query_installation::facade::WorthQueryArtifactCarriageContract::new(
        worth_query_installation::facade::WorthQueryArtifactMovePosture::Required,
        worth_query_installation::facade::WorthQueryArtifactBorrowPosture::SharedReadOnly,
        worth_query_installation::facade::WorthQueryArtifactClonePosture::Forbidden,
        worth_query_installation::facade::WorthQueryArtifactProviderTransferPosture::MoveOwnership,
        worth_query_installation::facade::WorthQueryArtifactSerializationPosture::CanonicalProjectionOnly,
    ))
    .lifecycle(WorthQueryArtifactLifecycleContract::ArenaScoped)
    .counters(WorthQueryStructuralCounterContract::required_foundation(
        counter("bytes"),
        counter("elements"),
        counter("work"),
    ))
    .decisions(WorthQueryDecisionRecordContract::not_required())
    .governance(
        worth_query_installation::facade::WorthQueryArtifactGovernanceContract::new(
            ["internal"],
            WorthQueryArtifactClassification::Internal,
            WorthQueryArtifactRedactionPosture::NotRequired,
            RetentionDeliveryProfile::Ephemeral,
            WorthQueryArtifactDeletionPosture::DeleteWithRun,
            WorthQueryArtifactLegalHoldPosture::NotEligible,
        ),
    )
    .compatibility(WorthQueryArtifactCompatibilityContract::new(
        WorthQueryArtifactCompatibilityWindow::new(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactSchemaVersion::new(2),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
        ),
        "WORTH.tests.affinity.migration",
        WorthQueryArtifactRetirementRule::Active,
        WorthQueryArtifactDowngradePosture::Denied,
    ))
    .produced_by(["producer"])
    .consumed_by(["consumer"])
    .finish()
    .unwrap()
}
pub fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
