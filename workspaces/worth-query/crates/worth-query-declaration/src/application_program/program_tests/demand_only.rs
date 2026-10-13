use super::*;
use crate::application_program::{ApplicationDemandOutputGraph, ApplicationSemanticFamily};

struct DemandProgram;
impl ApplicationProgramDefinition<TestSchema> for DemandProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<
        ApplicationDemandOutputGraph<FlatConnection, ApplicationOutputLeaf>,
    >;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ConnectedProgram::IDENTITY;
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        ConnectedProgram::feature_specs()
    }
}

#[test]
fn demand_only_root_changes_canonical_posture_without_rewriting_required_connections() {
    let required = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .unwrap();
    let demand = ApplicationProgramAuthoring::<TestSchema, DemandProgram>::begin()
        .validated_program()
        .unwrap();
    let required_manifest = required.normalized_manifest();
    let demand_manifest = demand.normalized_manifest();
    assert!(!required.connections()[0].is_demand_only_root());
    assert!(demand.connections()[0].is_demand_only_root());
    let extra = demand_manifest
        .records()
        .iter()
        .filter(|r| !required_manifest.records().contains(r))
        .collect::<Vec<_>>();
    assert_eq!(extra.len(), 1);
    assert!(extra[0].contains("demand-only"));
    assert!(extra[0].contains(Connection::IDENTITY));
    assert!(required_manifest
        .records()
        .iter()
        .all(|record| demand_manifest.records().contains(record)));
    assert_ne!(required.revision(), demand.revision());
    let changes = demand
        .semantic_description()
        .facts()
        .iter()
        .filter(|fact| fact.family() == ApplicationSemanticFamily::Connections)
        .count();
    assert_eq!(changes, 2);
}
