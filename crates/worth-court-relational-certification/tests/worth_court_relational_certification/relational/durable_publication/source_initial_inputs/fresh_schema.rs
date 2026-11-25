//! Original unused schema meaning is present in the actual Complete source DATA.
use super::*;
use worth_relational::facade::{config::CrossContextPolicy, identity::KindId, schema::*};
use worth_testkit_relational::durable_publication::durable_runtime;

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
fn source_plan(
    runtime: &RelationalRuntime,
    source: Arc<dyn RelationalDurableColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveryPlan {
    runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            RelationalDurableRecoverySource::new(None, vec![], vec![], source),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}

/// Restored C2 split of
/// 5d37a209ee:crates/worth-relational/src/durability/authority/fresh_source/tests.rs
/// (`fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`).
/// Fixture adaptations: the runtime owns a `TestDurableMedium` attachment (the
/// original used the retired in-crate medium); the unused relation kind is the
/// shared `installation()` fixture built from the same declaration; root and
/// schema `Arc` identities become `RelationalBasisCustodyWitness` identities;
/// the attached source identity is `reads_durable_source`; the private Complete
/// decode is the owner's `inspect_component_roster`. The public Complete wire
/// readback of both original schema roots and the unused constraint (formerly
/// a separate journey over the same inputs) is asserted here too.
#[test]
fn fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let born = current(&execution, &runtime);
    let additions = installation();
    let expected_constraint = additions
        .relation_registration(KindId(901))
        .unwrap()
        .relation_integrity
        .clone();
    let initialization = runtime.begin_fresh_home_initialization(&execution).unwrap();
    let installation = runtime
        .prepare_fresh_home_schema_installation(&execution, initialization)
        .unwrap()
        .install_with_managed_sources(&execution, additions, vec![], vec![])
        .unwrap();
    let recorded = current(&execution, &runtime);
    let id = runtime.runtime_instance_id();
    let identity = runtime.main_branch_identity();
    let original = runtime
        .observe_branch(&identity)
        .unwrap()
        .1
        .custody_witness();
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    assert!(runtime.reads_durable_source(&source));
    let plan = source_plan(&runtime, source.clone(), &execution);
    let admitted = runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .unwrap();
    admitted
        .validate_fresh_source_runtime(&execution, &runtime)
        .unwrap();
    runtime
        .validate_fresh_initial_schema_installation(&execution, &admitted, &installation)
        .unwrap();
    assert_eq!(runtime.runtime_instance_id(), id);
    assert!(runtime.reads_durable_source(&source));
    let actual = runtime
        .observe_branch(&identity)
        .unwrap()
        .1
        .custody_witness();
    assert!(original.same_root(&actual));
    assert!(original.same_schema_authority(&actual));
    let complete = worth_testkit_relational::initial_input_source::capture(&runtime, &execution);
    let roster = complete
        .checkpoint()
        .inspect_component_roster(
            &execution,
            worth_testkit_relational::cold_recovery::members_source(complete.checkpoint()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(roster.retained_empty_components, 1);
    assert!(roster
        .schema_registries
        .iter()
        .any(|registry| registry.authority_digest_bytes() == actual.schema_authority_digest()));
    assert!(
        roster.schema_registries.iter().any(|registry| registry
            .relation_registration(KindId(901))
            .is_ok_and(|kind| kind.relation_integrity == expected_constraint)),
        "unused physical constraint comes from original installation, not records"
    );
    let input = initial(complete.source_artifact());
    let schemas = input["schemas"].as_array().unwrap();
    let expected_wire =
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
            .any(|kind| kind["kind_id"] == 901 && kind["relation_integrity"] == expected_wire)),
        "Complete carries the unused original physical constraint without any record using it"
    );
    let foreign_medium = TestDurableMedium::new();
    let mut foreign = durable_runtime(&foreign_medium);
    let wrong = foreign
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    assert!(runtime
        .validate_fresh_initial_schema_installation(&execution, &admitted, &wrong)
        .is_err());
    assert!(admitted
        .validate_fresh_source_runtime(&execution, &foreign)
        .is_err());
}
