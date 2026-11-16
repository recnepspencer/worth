//! The authored indexed request really reaches its fixed output connection.
use super::super::super::prefix;
use super::super::super::{host, query, rule};
use super::super::{operation, source};
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
        let before = current
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let index = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "sh".into(),
            })
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
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
        let receipt = performed.receipt().clone();
        let publication = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        assert_eq!(publication.owner_identity(), before.owner_identity());
        assert_eq!(
            publication.ordinal(),
            before.ordinal() + 1,
            "source and required output have one physical final publication"
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &publication
        );
        let authored = current
            .query(source::ReadDirectory {
                directory: "A".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(authored.rows().len(), 1);
        assert_eq!(
            (&*authored.rows()[0].directory, &*authored.rows()[0].note),
            ("A", "indexed-request")
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &publication
        );
        let produced_before_delivery = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "produced:".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(produced_before_delivery.rows().len(), 1);
        assert_eq!(
            (
                &*produced_before_delivery.rows()[0].identity,
                &*produced_before_delivery.rows()[0].name
            ),
            ("managed-x-shared", "produced:shared")
        );
        // Separate reads own separate snapshot handles. Their selected product
        // version, runtime and branch must still name the same publication.
        let source_inspection = authored.receipt().inspect();
        let source_basis = source_inspection.basis();
        let output_inspection = produced_before_delivery.receipt().inspect();
        let output_basis = output_inspection.basis();
        assert_eq!(
            (
                output_basis.runtime_instance(),
                output_basis.branch(),
                output_basis.version(),
                output_basis.posture()
            ),
            (
                source_basis.runtime_instance(),
                source_basis.branch(),
                source_basis.version(),
                source_basis.posture()
            )
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &publication
        );
        let output_coordinate = produced_before_delivery.rows()[0].coordinate;
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
        assert_eq!(settlement.root_producer_contacts_in_this_demand(), 0);
        assert_eq!(settlement.work().producer_contact_count(), 0);
        assert_eq!(settlement.work().derived_publication_count(), 0);
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &publication
        );
        assert_eq!(
            output
                .receipt()
                .committed_product_publication()
                .composite_commit(),
            &publication
        );
        let original = settlement
            .performed_source()
            .expect("the actual performed source remains owned");
        assert_eq!(
            original
                .receipt()
                .retained_receipt()
                .unwrap()
                .committed_product_publication()
                .composite_commit(),
            &publication
        );
        assert_eq!(original.root_fulfillment().source_commit(), &publication);
        assert_eq!(original.root_fulfillment().output_commit(), &publication);
        assert_eq!(settlement.observation().selected_commit(), &publication);
        // Reuse retains actual typed output correspondence; a new producer receipt
        // is not promised for an already-current output.
        let correspondence = settlement
            .root_settlement()
            .outputs_of::<operation::Outputs>()
            .unwrap();
        let item = correspondence.entity::<operation::ItemOutput>().unwrap();
        let entity = item.entity_id();
        assert_eq!(
            (
                entity.partition_value(),
                entity.local_slot_value(),
                entity.generation_value()
            ),
            output_coordinate
        );
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
        assert_eq!(produced.rows(), produced_before_delivery.rows());
        let delivered_inspection = produced.receipt().inspect();
        let delivered_basis = delivered_inspection.basis();
        assert_eq!(
            (
                delivered_basis.runtime_instance(),
                delivered_basis.branch(),
                delivered_basis.version(),
                delivered_basis.posture()
            ),
            (
                output_basis.runtime_instance(),
                output_basis.branch(),
                output_basis.version(),
                output_basis.posture()
            )
        );
        drop((settlement, output, receipt, authored));
        index.close();
        assert_eq!(
            index.lifecycle(),
            worth_query_host::facade::primary_graph::WorthQueryManagedIndexLifecycle::Disposed
        );
        assert_eq!(
            produced.rows(),
            produced_before_delivery.rows(),
            "owned rows survive registration close"
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &publication
        );
    });
}
