use super::*;

struct ExpandedFlatProgram;
struct ReorderedConnectedProgram;

impl ApplicationProgramDefinition<TestSchema> for ExpandedFlatProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.flat-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .conditional_operation::<Operation>()
            .provides::<Export>()
            .finish()]
    }
}

impl ApplicationProgramDefinition<TestSchema> for ReorderedConnectedProgram {
    type Contributions = ();
    type Outputs =
        ApplicationProgramOutputs<ApplicationOutputGraph<FlatConnection, ApplicationOutputLeaf>>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.connected-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TestSchema, ConsumerFeature>().finish(),
            ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .provides::<Export>()
                .finish(),
        ]
    }
}

#[test]
fn normalized_manifest_is_order_independent_and_diagnostic_only() {
    let authored = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .unwrap();
    let reordered = ApplicationProgramAuthoring::<TestSchema, ReorderedConnectedProgram>::begin()
        .validated_program()
        .unwrap();

    assert_eq!(
        authored.normalized_manifest(),
        reordered.normalized_manifest()
    );
    assert_eq!(
        authored.normalized_manifest().program_identity(),
        "worth.query.tests.connected-program.v1"
    );
    assert!(authored
        .normalized_manifest()
        .records()
        .iter()
        .all(|record| !record.contains("TypeId")));
}

#[test]
fn compatible_port_addition_preserves_program_identity_and_existing_manifest_records() {
    let original = ApplicationProgramAuthoring::<TestSchema, FlatProgram>::begin()
        .validated_program()
        .unwrap()
        .normalized_manifest();
    let expanded = ApplicationProgramAuthoring::<TestSchema, ExpandedFlatProgram>::begin()
        .validated_program()
        .unwrap()
        .normalized_manifest();

    assert_eq!(
        original.program_identity(),
        "worth.query.tests.flat-program.v1"
    );
    assert_eq!(
        expanded.program_identity(),
        "worth.query.tests.flat-program.v1"
    );
    assert!(original
        .records()
        .iter()
        .all(|record| expanded.records().contains(record)));
    assert!(expanded.records().len() > original.records().len());
}
