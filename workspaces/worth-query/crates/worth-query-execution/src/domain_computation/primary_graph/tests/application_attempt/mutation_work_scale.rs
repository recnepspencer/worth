use std::collections::BTreeMap;

use worth_query_installation::facade::TypedApplicationValue;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use super::super::fixture::{
    Account, AccountIdentity, AccountLabel, AccountStatus, IdentityExecutionSchema,
    TouchAccountInput, TouchAccountOperation,
};
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitOutcome,
};

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
        baseline.proposed_fact_count() as u64
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
            work.preimage_candidates_materialized(),
            0,
            "{name} no-demand commit must collect no retention candidates"
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

fn mutation_work(
    unrelated_accounts: usize,
    idempotency_key: u8,
) -> super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(true);
    grow_unrelated_accounts(&world, unrelated_accounts);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(&world, &principal, &account, &request, "scale-committed");
    commit_work(&world, program, idempotency_key)
}

fn no_demand_mutation_work(
    wide: bool,
    idempotency_key: u8,
) -> super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let other = wide.then(|| resolved_account(&world, "unrelated", &request));
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
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
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
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
    commit_work(&world, effects.finish().unwrap(), idempotency_key)
}

fn commit_work(
    world: &super::super::fixture::AuthorizationWorld,
    program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
        IdentityExecutionSchema,
        TouchAccountOperation,
        TouchAccountInput,
        Account,
    >,
    idempotency_key: u8,
) -> super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence {
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(program, idempotency(idempotency_key, idempotency_key))
    else {
        panic!("mutation work fixture commits");
    };
    receipt
        .mutation_work()
        .expect("the returned receipt carries exact mutation work")
        .clone()
}

fn grow_unrelated_accounts(world: &super::super::fixture::AuthorizationWorld, count: usize) {
    if count == 0 {
        return;
    }
    let graph = world.application.primary_provider.graph.clone();
    let kind = graph
        .layout
        .entity_kind(Account::reference().name())
        .expect("account kind is installed");
    let locator = |entity: &str, aspect: &str, field: &str| {
        graph
            .layout
            .field_locator(entity, aspect, field)
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
    let batch = (0..count).fold(
        WorkerIntentBatch::new("unrelated-mutation-scale-population"),
        |batch, ordinal| {
            let key = format!("unrelated-scale-{ordinal}");
            let fields = AspectFieldPatch::from(BTreeMap::from([
                (identity.clone(), key.clone().into_foundational_value()),
                (
                    status.clone(),
                    "unrelated".to_owned().into_foundational_value(),
                ),
                (
                    label.clone(),
                    "population".to_owned().into_foundational_value(),
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
    super::super::fixture::publish_relational_mutation(world, batch);
}
