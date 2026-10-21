//! The collection and computation must be one checked declaration.
use super::*;

#[test]
fn collection_without_its_declared_computation_is_denied() {
    struct Missing;
    impl ApplicationProgramDefinition<TestSchema> for Missing {
        type Contributions = ();
        type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("missing-collection-computation");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .derived_collection::<Collection>()
                .finish()]
        }
    }
    let denied = ApplicationProgramAuthoring::<TestSchema, Missing>::begin()
        .validated_program()
        .err()
        .expect("collection computation must be declared");
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::MissingCollectionComputation
    );
}

struct SerialCollection;
impl ApplicationDerivedCollection<TestSchema, FlatFeature> for SerialCollection {
    type Computation = SerialComputation;
    type Completeness = Completeness;
    type Contributor = Contributor;
    type Grouping = Grouping;
    type Measures = Measures;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "serial-collection";
    const APPLICABILITY: &'static str = Collection::APPLICABILITY;
    const INCOMPLETE: ApplicationCollectionIncompletePosture = Collection::INCOMPLETE;
    const INCREMENTAL_UPDATE: &'static str = Collection::INCREMENTAL_UPDATE;
}

#[test]
fn collection_cannot_bind_a_whole_input_computation() {
    struct SerialProgram;
    impl ApplicationProgramDefinition<TestSchema> for SerialProgram {
        type Contributions = ();
        type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("serial-collection-program");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .derived_artifact::<Artifact>()
                .managed_computation::<SerialComputation>()
                .derived_collection::<SerialCollection>()
                .finish()]
        }
    }
    let denied = ApplicationProgramAuthoring::<TestSchema, SerialProgram>::begin()
        .validated_program()
        .err()
        .expect("collection needs keyed execution");
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::CollectionComputationNotPartitioned
    );
}

#[test]
fn canonical_manifest_names_the_bound_computation_and_completeness() {
    let program = ApplicationProgramAuthoring::<TestSchema, Program>::begin()
        .validated_program()
        .unwrap();
    let collection = program.features()[0].derived_collections()[0];
    assert_eq!(
        collection.computation_type(),
        std::any::TypeId::of::<Computation>()
    );
    assert_eq!(
        collection.completeness_type(),
        std::any::TypeId::of::<Completeness>()
    );
    assert_eq!(collection.value_type(), std::any::TypeId::of::<u64>());
    let manifest = program.normalized_manifest();
    let record = manifest
        .records()
        .iter()
        .find(|record| record.contains(Collection::IDENTITY))
        .unwrap();
    assert!(record.contains(Computation::IDENTITY));
    assert!(record.contains(
        <Completeness as crate::application_program::ApplicationCollectionCompleteness>::IDENTITY
    ));
}
