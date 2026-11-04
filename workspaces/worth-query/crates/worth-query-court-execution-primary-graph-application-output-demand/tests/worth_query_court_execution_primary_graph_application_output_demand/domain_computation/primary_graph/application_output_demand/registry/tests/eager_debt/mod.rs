//! An independent eager model observes every step, including admission-free reads.
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::model::Custody;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::model::Model;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::model::Phase;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::model::Row;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::World;

#[test]
fn registry_eager_debt_oracle_checks_every_step_for_32_seeds() {
    let context = crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context::new();
    let mut divergences = Vec::new();
    for seed in 0..32 {
        crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::driver::run(&context, seed, &mut divergences);
        crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::unhold::run(&context, seed, &mut divergences);
    }
    for divergence in &divergences {
        println!("{divergence}");
    }
    assert!(divergences.is_empty(), "eager equivalence is still red");
}

#[test]
fn registry_eager_binding_reads_the_closed_claim_without_an_admission() {
    let context = crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context::new();
    let mut world = World::new(&context, 1);
    world.drop_owner(1);
    world.drop_owner(2);
    world.bind();
    let state = (*world.registry.state_for_test()).lock().unwrap();
    let bound = (*state.source_custody[&context.commit(0)].bound_sources_for_test()).clone();
    assert_eq!(
        bound,
        Some(vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(context.scope, (*world.keys[0].source_for_test()).clone())])
    );
    assert!((*state.source_custody[&context.commit(0)].retired_sources_for_test()).is_empty());
}

#[test]
fn registry_eager_claimed_row_transition_itself_leaves_the_note() {
    let context = crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context::new();
    let world = World::new(&context, 3);
    let mut state = (*world.registry.state_for_test()).lock().unwrap();
    assert_eq!(
        (*(*state.records_for_test())[&world.keys[0]].framework_required_count_for_test()),
        1
    );
    (*(*state.records_mut_for_test())
        .get_mut(&world.keys[0])
        .unwrap()
        .state_mut_for_test()) = DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::Superseded,
        "transition",
    ));
    assert!(
        (*(*state.records_for_test())[&world.keys[0]].closed_cleanup_for_test()).queued_for_test()
    );
}

#[test]
fn registry_eager_owned_closed_note_does_not_name_a_reused_key() {
    let context = crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context::new();
    let registry = WorthQueryOutputDemandRegistry::default();
    let reused = key("reused-note", 1, 90);
    let old = {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let old = record(
            occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::Closed,
                "old",
            )),
            0,
        );
        (*state.records_mut_for_test()).insert_for_test(reused.clone(), old);
        state.note_closed_row_for_test(&reused);
        let old = state.remove_record(&reused).unwrap();
        let mut new = record(occurrence(), DemandState::Admitted, 0);
        (*new.source_scope_mut_for_test()) = Some(context.scope);
        (*state.records_mut_for_test()).insert_for_test(reused.clone(), new);
        old
    };
    let owner = registry
        .admit_for_test(
            reused.clone(),
            None,
            context.scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .unwrap();
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!(
            (*(*state.records_for_test())[&reused].interests_for_test()),
            1
        );
        assert_eq!((*state.closed_cleanup_for_test()).len_for_test(), 0);
    }
    drop(old); // Even an original row kept outside the map cannot name its replacement.
    drop(owner);
}

#[path = "driver/mod.rs"]
pub mod driver;
#[path = "foreign/mod.rs"]
pub mod foreign;
#[path = "model/mod.rs"]
pub mod model;
#[path = "observe/mod.rs"]
pub mod observe;
#[path = "unhold/mod.rs"]
pub mod unhold;
#[path = "world/mod.rs"]
pub mod world;
