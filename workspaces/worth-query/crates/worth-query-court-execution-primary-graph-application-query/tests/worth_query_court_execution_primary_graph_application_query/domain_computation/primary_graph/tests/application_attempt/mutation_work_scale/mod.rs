pub use std::collections::BTreeMap;

pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::CreateIntent;
pub use worth_relational::facade::transactions::EntitySpec;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn mutation_work_is_invariant_to_unrelated_graph_population() {
    let baseline = mutation_work(0, 71);
    let expanded = mutation_work(128, 72);

    assert_eq!(
        expanded.decision_fact_count(),
        baseline.decision_fact_count()
    );
    assert_eq!(
        expanded.proposed_fact_count(),
        baseline.proposed_fact_count()
    );
    assert_eq!(
        expanded.expected_step_key_lookups(),
        baseline.expected_step_key_lookups()
    );
    assert_eq!(
        baseline.expected_step_key_lookups(),
        baseline.proposed_fact_count()
    );
    assert_eq!(baseline.expected_step_duplicate_equalities(), 0);
    assert_eq!(
        expanded.invariant_state_fact_count(),
        baseline.invariant_state_fact_count()
    );
    assert_eq!(
        expanded.invariant_work_units(),
        baseline.invariant_work_units()
    );
    assert_eq!(
        expanded.relational_invariant_execution_count(),
        baseline.relational_invariant_execution_count()
    );
    assert_eq!(
        expanded.relational_invariant_result_count(),
        baseline.relational_invariant_result_count()
    );
    assert_eq!(
        expanded.performed_touch_validated_intents_examined(),
        baseline.performed_touch_validated_intents_examined()
    );
    assert_eq!(
        expanded.performed_touch_targets_materialized(),
        baseline.performed_touch_targets_materialized()
    );
    assert_eq!(
        expanded.performed_application_touches_admitted(),
        baseline.performed_application_touches_admitted()
    );
    assert_eq!(
        expanded.installed_touch_scopes_compared(),
        baseline.installed_touch_scopes_compared()
    );
    assert_eq!(
        expanded.installed_read_touch_overlaps_observed(),
        baseline.installed_read_touch_overlaps_observed()
    );
    assert!(baseline.decision_fact_count() > 0);
    assert!(baseline.proposed_fact_count() > 0);
    assert_eq!(
        baseline.invariant_state_fact_count(),
        baseline.proposed_fact_count()
    );
    assert_eq!(
        baseline.invariant_work_units(),
        // The always-installed publication custody receipt costs one fixed
        // unit even when this account mutation does not touch a workflow.
        baseline.proposed_fact_count() as u64 + 1
    );
    assert_eq!(baseline.relational_invariant_execution_count(), 3);
    assert!(baseline.relational_invariant_result_count() > 0);
    assert!(baseline.performed_touch_validated_intents_examined() > 0);
    assert!(baseline.performed_touch_targets_materialized() > 0);
    assert_eq!(baseline.performed_application_touches_admitted(), 1);
    assert!(baseline.installed_touch_scopes_compared() >= 1);
    assert!(baseline.installed_read_touch_overlaps_observed() >= 1);
    for (name, work) in [("baseline", &baseline), ("expanded", &expanded)] {
        assert_eq!(
            work.preimage_validated_intents_examined(),
            0,
            "{name} no-demand commit must not ask Relational for a footprint"
        );
        assert_eq!(
            work.preimage_mutation_targets_materialized(),
            0,
            "{name} no-demand commit must materialize no footprint targets"
        );
        assert_eq!(
            work.preimage_decision_facts_examined(),
            0,
            "{name} no-demand commit must scan no decision facts"
        );

        assert_eq!(
            work.preimage_demanded_loci_examined(),
            0,
            "{name} no-demand commit must examine no pre-image loci"
        );
    }
    // C2 — commit-derived names are present. Absolute EntityIds differ across
    // separately populated worlds; the *count* of records this mutation
    // touched must not grow with unrelated graph width.
    assert!(!baseline.touched_records().is_empty());
    assert_eq!(
        expanded.touched_record_count(),
        baseline.touched_record_count()
    );
}

#[test]
fn no_demand_work_is_exact_zero_across_real_mutation_breadth() {
    let narrow = no_demand_mutation_work(false, 73);
    let wide = no_demand_mutation_work(true, 74);

    assert_eq!(narrow.proposed_fact_count(), 1);
    assert_eq!(wide.proposed_fact_count(), 2);
    assert_eq!(narrow.expected_step_key_lookups(), 1);
    assert_eq!(wide.expected_step_key_lookups(), 2);
    assert!(wide.touched_record_count() > narrow.touched_record_count());
    assert_eq!(narrow.performed_application_touches_admitted(), 1);
    assert_eq!(wide.performed_application_touches_admitted(), 2);
    assert_eq!(
        narrow.installed_touch_scopes_compared(),
        wide.installed_touch_scopes_compared()
    );
    for (name, work) in [("narrow", &narrow), ("wide", &wide)] {
        assert_eq!(work.preimage_validated_intents_examined(), 0, "{name}");
        assert_eq!(work.preimage_mutation_targets_materialized(), 0, "{name}");
        assert_eq!(work.preimage_decision_facts_examined(), 0, "{name}");
        assert_eq!(work.preimage_candidates_materialized(), 0, "{name}");
        assert_eq!(work.preimage_demanded_loci_examined(), 0, "{name}");
    }
}

pub fn mutation_work(
    unrelated_accounts: usize,
    idempotency_key: u8,
) -> worth_query_execution::facade::domain_computation::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let phase_custody = &custody;
            grow_unrelated_accounts(
                &phase_custody.borrowed_request_for_test(),
                &world,
                unrelated_accounts,
            );
            let request = live_scope();
            let principal = authenticated_principal(
                &(phase_custody).test_host_request_for_test(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(phase_custody).test_host_request_for_test(),
                &world,
                "open",
                &request,
            );

            admitted_program(
                &phase_custody.phase_for_application_for_test(&world.application),
                &world,
                &principal,
                &account,
                &request,
                "scale-committed",
            )
        })
        .expect("the fixture preparation owns its host call");
    commit_work(&world, program, idempotency_key)
}

pub fn no_demand_mutation_work(
    wide: bool,
    idempotency_key: u8,
) -> worth_query_execution::facade::domain_computation::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = world
        .application
        .with_host_advancement(|phase| {
            no_demand_mutation_program(&phase.test_host_request_for_test(), &world, wide)
        })
        .expect("the fixture preparation owns its host call");
    commit_work(&world, program, idempotency_key)
}

pub fn no_demand_mutation_program(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    wide: bool,
) -> worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
> {
    let request = live_scope();
    let principal = authenticated_principal(
        &(phase_custody).test_host_request_for_test(),
        world,
        &request,
    );
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "open",
        &request,
    );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let other = wide.then(|| {
        resolved_account(
            &(phase_custody).test_host_request_for_test(),
            world,
            "unrelated",
            &request,
        )
    });
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::reference())
                    .unwrap();
                if wide {
                    let other = reader
                        .resolve_entity(AccountStatus::reference(), "unrelated".to_owned())
                        .unwrap();
                    reader
                        .require_decision_field(&other, AccountLabel::reference())
                        .unwrap();
                }
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountStatus::reference(), "breadth".to_owned())
        .unwrap();
    if let Some(other) = other {
        let other = effects.existing_entity(&other).unwrap();
        effects
            .write_field(&other, AccountLabel::reference(), "wide".to_owned())
            .unwrap();
    }
    effects.finish().unwrap()
}

pub fn commit_work(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    program: worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram<
        IdentityExecutionSchema,
        TouchAccountOperation,
        TouchAccountInput,
        Account,
    >,
    idempotency_key: u8,
) -> worth_query_execution::facade::domain_computation::WorthQueryPrimaryMutationWorkEvidence {
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(idempotency_key, idempotency_key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("mutation work fixture commits");
    };
    receipt
        .mutation_work()
        .expect("the returned receipt carries exact mutation work")
        .clone()
}

pub fn grow_unrelated_accounts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    count: usize,
) {
    if count == 0 {
        return;
    }
    let graph = (*(*(*world.application.runtime_for_test()).primary_provider_for_test()).graph_for_test()).clone();
    let kind = (*graph
        .layout_for_test())
        .entity_kind_for_test(Account::reference().name())
        .expect("account kind is installed");
    let locator = |entity: &str, aspect: &str, field: &str| {
        (*graph.layout_for_test())
            .field_locator_for_test(entity, aspect, field)
            .expect("scale field is installed")
            .clone()
    };
    let identity_ref = AccountIdentity::reference();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let identity = locator(
        identity_ref.entity(),
        identity_ref.aspect(),
        identity_ref.field(),
    );
    let status = locator(status_ref.entity(), status_ref.aspect(), status_ref.field());
    let label = locator(label_ref.entity(), label_ref.aspect(), label_ref.field());
    for start in (0..count).step_by(4_000) {
        let batch = (start..(start + 4_000).min(count)).fold(
            WorkerIntentBatch::new("unrelated-mutation-scale-population"),
            |batch, ordinal| {
                let key = format!("unrelated-scale-{ordinal}");
                let fields = AspectFieldPatch::from(BTreeMap::from([
                    (
                        identity.clone(),
                        StringApplicationValueBinding::encode(&key.clone()).unwrap(),
                    ),
                    (
                        status.clone(),
                        StringApplicationValueBinding::encode(&"population".to_owned()).unwrap(),
                    ),
                    (
                        label.clone(),
                        StringApplicationValueBinding::encode(&"population".to_owned()).unwrap(),
                    ),
                ]));
                batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: kind,
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key),
                    fields,
                })))
            },
        );
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(execution, world, batch);
    }
}

#[path = "completed_evidence/mod.rs"]
pub mod completed_evidence;
#[path = "locality/mod.rs"]
pub mod locality;
