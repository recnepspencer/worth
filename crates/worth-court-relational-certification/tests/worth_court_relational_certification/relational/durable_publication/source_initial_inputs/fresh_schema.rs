//! Original unused schema meaning is present in the actual Complete source DATA.
use super::*;
use worth_relational::facade::{config::CrossContextPolicy, identity::KindId, schema::*};

fn installation() -> RelationalSchemaRegistry {
    let mut unused = worth_testkit_relational::durable_publication::schema()
        .relation_registration(KindId(2))
        .unwrap()
        .clone();
    unused.kind_id = KindId(901);
    unused.kind_name = "unused.fresh.physical".into();
    unused.aspect_contract_declarations = Default::default();
    unused.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "unused.fresh.no.self.edge".into(),
            allowed_source_kinds: vec![KindId(1)],
            allowed_target_kinds: vec![KindId(1)],
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    RelationalSchemaRegistry::new()
        .register_relation_kind(unused)
        .unwrap()
}
#[test]
fn fresh_source_keeps_original_installation_receipts_and_unused_constraints_in_complete() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = worth_testkit_relational::durable_publication::durable_runtime(&medium);
    let born = current(&request, &runtime);
    let additions = installation();
    let expected_constraint = additions
        .relation_registration(KindId(901))
        .unwrap()
        .relation_integrity
        .clone();
    let permission = runtime.begin_fresh_home_initialization(&request).unwrap();
    let installed = runtime
        .prepare_fresh_home_schema_installation(&request, permission)
        .unwrap()
        .install_with_managed_sources(&request, additions, vec![], vec![])
        .unwrap();
    let recorded = current(&request, &runtime);
    let id = runtime.runtime_instance_id();
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            &request,
            RelationalDurableRecoverySource::new(None, vec![], vec![], source),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let admitted = runtime
        .durability_recovery()
        .admit_fresh_home_source(&request, plan)
        .unwrap();
    admitted
        .validate_fresh_source_runtime(&request, &runtime)
        .unwrap();
    runtime
        .validate_fresh_initial_schema_installation(&request, &admitted, &installed)
        .unwrap();
    assert_eq!(runtime.runtime_instance_id(), id);
    let complete = worth_testkit_relational::initial_input_source::capture(&runtime, &request);
    let input = initial(complete.source_artifact());
    let schemas = input["schemas"].as_array().unwrap();
    let expected_constraint =
        crate::source_wire_value::read(&rmp_serde::to_vec_named(&expected_constraint).unwrap());
    for original in [&born, &recorded] {
        assert!(schemas.iter().any(|schema| schema["schema_root"]
            == serde_json::to_value(original.descriptor().schema_commitment()).unwrap()));
    }
    assert!(
        schemas.iter().any(|schema| schema["relation_registrations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|kind| {
                kind["kind_id"] == 901 && kind["relation_integrity"] == expected_constraint
            })),
        "Complete carries the unused original physical constraint without any record using it"
    );
    let foreign_medium = TestDurableMedium::new();
    let mut foreign =
        worth_testkit_relational::durable_publication::durable_runtime(&foreign_medium);
    let wrong = foreign
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    assert!(runtime
        .validate_fresh_initial_schema_installation(&request, &admitted, &wrong)
        .is_err());
    assert!(admitted
        .validate_fresh_source_runtime(&request, &foreign)
        .is_err());
}
