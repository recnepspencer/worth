use worth_foundational::facade::{
    CanonicalizationRuleVersion, FoundationalPerformanceCounterName, RetentionDeliveryProfile,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};

use crate::application::{WorthQueryCapabilityFamily, WorthQueryDomainEntryMarker};

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArtifactContractDomain;

impl WorthQueryDomainEntryMarker for ArtifactContractDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.artifact-contract"
    }

    fn display_name(&self) -> &'static str {
        "Artifact Contract"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[]
    }
}

struct CandidateArtifact;
struct PackageSchema;
struct PackageEntity;

impl ApplicationEntityMarkerIdentity<PackageSchema> for PackageEntity {
    const IDENTIFIER: &'static str = "PackageEntity";
}

impl ApplicationSchema for PackageSchema {
    const OWNER: &'static str = "WORTH.tests.artifact-contract";
    const NAME: &'static str = "PackageSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(
                ApplicationEntityRef::<Self, PackageEntity>::from_schema_identifier(
                    "PackageEntity",
                ),
            )
            .build()
    }
}

impl WorthQueryArtifactFamily for CandidateArtifact {
    const SEMANTIC_FAMILY: &'static str = "WORTH.tests.artifact-contract.candidates";
}

#[test]
fn typed_package_validation_carries_artifact_contract_into_portable_meaning() {
    let contract = candidate_contract();
    let package = WorthQueryDomainPackage::declare(
        ArtifactContractDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new("artifact-contract").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .artifact_contract(contract.clone())
    .validate()
    .unwrap();

    assert_eq!(package.portable_package.artifact_contracts(), &[contract]);
}

#[test]
fn typed_package_validation_carries_application_schema_into_portable_meaning() {
    let package = WorthQueryDomainPackage::declare(
        ArtifactContractDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new("artifact-contract").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .application_schema(PackageSchema::declaration().unwrap())
    .validate()
    .unwrap();

    let schemas = package.portable_package.application_schemas();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].name(), PackageSchema::NAME);
    assert_eq!(
        schemas[0].identity(),
        PackageSchema::declaration().unwrap().identity()
    );
}

fn candidate_contract() -> WorthQueryPortableArtifactContract {
    WorthQueryPortableArtifactContract::declare::<CandidateArtifact>(
        WorthQueryArtifactSchemaVersion::new(1),
        WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "WORTH.tests.artifact-contract.projection",
            CanonicalizationRuleVersion::new("artifact-contract-v1").unwrap(),
        ),
    )
    .ownership(WorthQueryArtifactOwnershipContract::domain_payload(
        "WORTH.tests.artifact-contract",
        "WORTH.tests.artifact-contract.provider",
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
        WorthQueryArtifactComparisonAuthority::ExactCanonicalValue,
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    ))
    .search(WorthQueryCandidateSearchContract::not_applicable())
    .convergence(WorthQueryConvergenceContract::NotIterative)
    .transformation(WorthQueryTransformationEvidenceContract::not_a_transformation())
    .access_path(WorthQueryArtifactAccessPathContract::denied())
    .carriage(WorthQueryArtifactCarriageContract::move_only_provider_transfer())
    .lifecycle(WorthQueryArtifactLifecycleContract::ArenaScoped)
    .counters(WorthQueryStructuralCounterContract::required_foundation(
        counter("bytes"),
        counter("elements"),
        counter("work"),
    ))
    .decisions(WorthQueryDecisionRecordContract::not_required())
    .governance(WorthQueryArtifactGovernanceContract::new(
        ["workflow-internal"],
        WorthQueryArtifactClassification::Internal,
        WorthQueryArtifactRedactionPosture::NotRequired,
        RetentionDeliveryProfile::Ephemeral,
        WorthQueryArtifactDeletionPosture::DeleteWithRun,
        WorthQueryArtifactLegalHoldPosture::NotEligible,
    ))
    .compatibility(WorthQueryArtifactCompatibilityContract::new(
        WorthQueryArtifactCompatibilityWindow::new(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
        ),
        "WORTH.tests.artifact-contract.migration",
        WorthQueryArtifactRetirementRule::Active,
        WorthQueryArtifactDowngradePosture::Denied,
    ))
    .produced_by(["producer"])
    .consumed_by(["consumer"])
    .finish()
    .unwrap()
}

fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
