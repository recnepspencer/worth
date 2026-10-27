//! Pure source meaning cannot install an absent proposed-state runtime owner.
use super::*;
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelation, ApplicationDerivedRelationRef, ApplicationRelationGraphLaws,
};

struct DerivedSchema;
struct TwoSteps;
impl OperationReads<TestOperation<DerivedSchema>> for TwoSteps {}

impl ApplicationSchema for DerivedSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "DerivedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let edge = ApplicationRelationRef::<Self, MappingTarget, FixtureEntity<Self>, FixtureEntity<Self>>::from_schema_identifiers(
            "MappingTarget", "TestEntity", "TestEntity",
            worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        );
        let derived = ApplicationDerivedRelationRef::<
            Self,
            TwoSteps,
            FixtureEntity<Self>,
            FixtureEntity<Self>,
        >::from_schema_identifiers("TwoSteps", "TestEntity", "TestEntity");
        test_schema_members::<Self>(None)
            .derived_relation(
                derived,
                ApplicationDerivedRelation::compose(edge, edge).without_self_edges(),
                ApplicationRelationGraphLaws::unbounded().acyclic(),
            )
            .operation_read_derived_relation(
                ApplicationOperationRef::<Self, TestOperation<Self>, TestInput>::from_declaration(),
                derived,
            )
            .rule_readable_collection::<Feature, Collection, _, _, _>(derived)
            .build()
    }
}

#[test]
fn derived_source_meaning_refuses_real_installation_until_native_owner_exists() {
    let declaration = DerivedSchema::declaration().unwrap();
    let association = declaration.erased().members().iter().find_map(|member| {
        let worth_query_declaration::facade::application_schema::ApplicationSchemaMember::RuleReadableCollection {association} = member else { return None };
        Some(association)
    }).expect("actual typed collection association");
    assert_eq!(
        (
            association.relation.as_str(),
            association.feature.as_str(),
            association.feature_major,
            association.feature_minor
        ),
        ("TwoSteps", "derived.fixture.feature", 2, 3)
    );
    assert_eq!(
        (
            association.collection.as_str(),
            association.computation.as_str(),
            association.output_artifact.as_str(),
            association.completeness.as_str()
        ),
        (
            "derived.fixture.collection",
            "derived.fixture.computation",
            "derived.fixture.artifact",
            "derived.fixture.completeness"
        )
    );
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(DerivedSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    let denial = index
        .bind_application_schema(declaration)
        .err()
        .expect("descriptive source cannot supply Native rule authority");
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::DerivedSourceUnavailable
    );
    assert_eq!(denial.subject(), "TwoSteps");
    // The ordinary installed schema still obtains its actual runtime binding.
    installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
}

struct Feature;
struct Collection;
struct Computation;
struct Artifact;
struct Output;
struct Contract;

impl ApplicationFeature<DerivedSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "derived.fixture.feature";
    const MAJOR: u16 = 2;
    const MINOR: u16 = 3;
}
impl ApplicationOutputPort<DerivedSchema, Feature> for Output {
    type Value = TestInputBinding;
    const IDENTITY: &'static str = "derived.fixture.output";
}
impl ApplicationLocalityScope for Contract {
    const IDENTITY: &'static str = "derived.fixture.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Neighborhood;
}
impl ApplicationDerivedArtifact<DerivedSchema, Feature> for Artifact {
    type Output = Output;
    type Locality = Contract;
    const IDENTITY: &'static str = "derived.fixture.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "derived.fixture.producer";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "derived.fixture.reuse";
    const STOPPED_OUTCOME: &'static str = "derived.fixture.stopped";
}
impl ApplicationComputationInput for Contract {
    type Value = ();
    const IDENTITY: &'static str = "derived.fixture.input";
}
impl ApplicationComputationReuse for Contract {
    const IDENTITY: &'static str = "derived.fixture.reuse";
}
impl ApplicationComputationStopped for Contract {
    const IDENTITY: &'static str = "derived.fixture.stopped";
}
impl ApplicationManagedComputation<DerivedSchema, Feature> for Computation {
    type Input = Contract;
    type Output = Artifact;
    type Partition = ApplicationSingleComputationPartition;
    type Reuse = Contract;
    type Stopped = Contract;
    const IDENTITY: &'static str = "derived.fixture.computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::Deterministic;
}
impl ApplicationCollectionContributor for Contract {
    const IDENTITY: &'static str = "derived.fixture.contributor";
}
impl ApplicationCollectionGrouping for Contract {
    const IDENTITY: &'static str = "derived.fixture.grouping";
}
impl ApplicationCollectionMeasures for Contract {
    const IDENTITY: &'static str = "derived.fixture.measures";
}
impl ApplicationCollectionLineage for Contract {
    const IDENTITY: &'static str = "derived.fixture.lineage";
}
impl ApplicationCollectionCompleteness for Contract {
    type Value = TestInput;
    type Stopped = ();
    type Lineage = Contract;
    const IDENTITY: &'static str = "derived.fixture.completeness";
    fn check(_: &TestInput) -> Result<(), ()> {
        Ok(())
    }
}
impl ApplicationDerivedCollection<DerivedSchema, Feature> for Collection {
    type Computation = Computation;
    type Completeness = Contract;
    type Contributor = Contract;
    type Grouping = Contract;
    type Measures = Contract;
    type Lineage = Contract;
    const IDENTITY: &'static str = "derived.fixture.collection";
    const APPLICABILITY: &'static str = "derived.fixture.applicability";
    const INCOMPLETE: ApplicationCollectionIncompletePosture =
        ApplicationCollectionIncompletePosture::RetainPending;
    const INCREMENTAL_UPDATE: &'static str = "derived.fixture.invalidation";
}
