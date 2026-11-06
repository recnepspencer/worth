//! The authored indexed request really reaches its fixed output connection.
use super::super::super::{host, query, rule};
use super::{program, request};
use crate::application_invariant_acceptance::{
    authentication,
    managed_index::{lifecycle, prefix_observation},
};
use std::sync::Arc;
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};

#[test]
fn indexed_request_action_publishes_its_fixed_managed_output() {
    lifecycle::run(|| {
        let provider = Arc::new(prefix_observation::Calls::default());
        let world = host::install(provider.clone(), Arc::new(rule::Calls::new(provider)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let current = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let outcome = current
            .advanced()
            .mutate(request::Request(
                super::super::source::operation::RequestItem {
                    directory: "A".into(),
                    name: "indexed-request".into(),
                },
            ))
            .without_source()
            .idempotency(&0x919a_8500u64)
            .execute_performed::<host::Program, program::Root>(
                &world,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
            panic!("the fresh indexed source action must commit");
        };
        let mut output = performed
            .start_required_outputs(&world, &current, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|failure| panic!("fixed indexed connection: {:?}", failure.denial()));
        let settlement = (0..64)
            .find_map(|_| {
                match output
                    .required_output_mut()
                    .advance(&world, &current)
                    .unwrap()
                {
                    WorthQueryApplicationProgramOutputProgress::Settled(settlement) => {
                        Some(settlement)
                    }
                    _ => None,
                }
            })
            .expect("the genuine indexed producer settles within the fixture bound");
        assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
        assert!(settlement.root_receipt().is_some());
        let produced = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "produced:".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(produced.rows().len(), 1);
        assert_eq!(
            (&*produced.rows()[0].identity, &*produced.rows()[0].name),
            ("managed-x-shared", "produced:shared")
        );
    });
}
