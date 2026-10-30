//! Complete derived meaning binds; the closed count Input stays entity-rooted.
use super::*;
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelation, ApplicationDerivedRelationRef, ApplicationRelationGraphLaws,
};

struct DerivedSchema<const LOGICAL: bool = false>;
struct Uses;
struct Side;
struct TwoSteps;
impl<const L: bool> OperationReads<TestOperation<DerivedSchema<L>>> for TwoSteps {}

impl<const L: bool> ApplicationSchema for DerivedSchema<L> {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "DerivedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        if L {
            use worth_query_declaration::facade::application_schema::{
                ApplicationProjectedMemberCaseRef, ApplicationProjectedMemberDomainDefinition,
                ApplicationProjectedMemberDomainRef,
            };
            let carrier = ApplicationEntityRef::<Self, FixtureEntity<Self>>::from_schema_identifier(
                "TestEntity",
            );
            let domain =
                ApplicationProjectedMemberDomainRef::<Self, Uses, FixtureEntity<Self>>::new(
                    "Uses", carrier,
                );
            let side = ApplicationProjectedMemberCaseRef::<Self, Uses, Side>::new(domain, "Side");
            let relation = ApplicationDerivedRelationRef::<Self, TwoSteps, Uses, Uses>::projected(
                "TwoSteps", domain,
            );
            return test_schema_members::<Self>(None)
                .projected_member_domain(ApplicationProjectedMemberDomainDefinition::new(domain).case(side).unwrap())
                .derived_relation(relation,ApplicationDerivedRelation::carrier_case(domain,side).unwrap().reversed().then(ApplicationDerivedRelation::carrier_case(domain,side).unwrap()),ApplicationRelationGraphLaws::unbounded())
                .operation_read_derived_relation(ApplicationOperationRef::<Self,TestOperation<Self>,TestInput>::from_declaration(),relation)
                .rule_readable_collection::<Feature,Collection,_,_,_>(relation).build();
        }
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
fn complete_derived_law_binds_real_installation_with_an_ordinary_twin() {
    let declaration = DerivedSchema::<false>::declaration().unwrap();
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
    .application_schema(DerivedSchema::<false>::declaration().unwrap())
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
    index.bind_application_schema(declaration).unwrap();
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

impl<const L: bool> ApplicationFeature<DerivedSchema<L>> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "derived.fixture.feature";
    const MAJOR: u16 = 2;
    const MINOR: u16 = 3;
}
impl<const L: bool> ApplicationOutputPort<DerivedSchema<L>, Feature> for Output {
    type Value = TestInputBinding;
    const IDENTITY: &'static str = "derived.fixture.output";
}
impl ApplicationLocalityScope for Contract {
    const IDENTITY: &'static str = "derived.fixture.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Neighborhood;
}
impl<const L: bool> ApplicationDerivedArtifact<DerivedSchema<L>, Feature> for Artifact {
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
impl<const L: bool> ApplicationManagedComputation<DerivedSchema<L>, Feature> for Computation {
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
impl<const L: bool> ApplicationDerivedCollection<DerivedSchema<L>, Feature> for Collection {
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

#[test]
fn logical_root_closed_count_refuses_its_actual_input_limitation() {
    let declaration = DerivedSchema::<true>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(DerivedSchema::<true>::declaration().unwrap())
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
        .expect_err("closed Input requires a genuine EntityIdentity");
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::DerivedSourceUnavailable
    );
    assert_eq!(denial.subject(), "TwoSteps");
}
