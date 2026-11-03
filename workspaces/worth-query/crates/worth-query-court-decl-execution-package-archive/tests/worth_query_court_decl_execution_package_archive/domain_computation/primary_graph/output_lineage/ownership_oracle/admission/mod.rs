//! Active native court, using actual installed publisher and exact performed row.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::OwnerSchema;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding;
use worth_relational::facade::mvcc::RequiredOutputPublicationMeaning;
use worth_relational::facade::mvcc::RequiredOutputRolePosture;
use worth_relational::facade::mvcc::RequiredOutputRoleTarget;
use worth_relational::facade::transactions::EntityReference;

pub fn refuse_mixed_and_created_subset(
    runtime: &Application,
    settled: &WorthQueryOutputDemandSettlement,
) {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime.with_application_advancement(&scope, |phase| {
        let execution = phase.execution_request_for_for_test(&(*runtime.product_runtime_for_test())).unwrap();
        let (_, product, _application_basis) = runtime.on_branch(runtime.current_world()).select(&phase).unwrap().into_parts_for_test();
        let graph = &(*(*runtime.primary_provider_for_test()).graph_for_test());
        let publisher = graph.application_publication.as_ref().expect("actual installed publication owner");
        let roles: Vec<_> = settled.output_correspondence.native_witness_roles_for_test().map(|(role, posture, name, entity)| RequiredOutputRoleTarget {
            role: role.to_owned(), entity_name: name.to_owned(), kind: (*graph.layout_for_test()).entity_kind_for_test(name).unwrap(),
            posture: match posture {
                WorthQueryApplicationOutputPosture::Create => RequiredOutputRolePosture::Create,
                WorthQueryApplicationOutputPosture::Preserve => RequiredOutputRolePosture::Preserve,
                WorthQueryApplicationOutputPosture::Retire => panic!("MODE2 has no retirement"),
            }, target: EntityReference::Existing(entity),
        }).collect();
        assert_eq!(roles.len(), 4);
        let anchor = roles.iter().find(|role| role.posture == RequiredOutputRolePosture::Preserve).unwrap();
        let EntityReference::Existing(anchor) = anchor.target else { unreachable!() };
        let lineage = graph.output_lineage.lock().unwrap();
        let source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource {
            runtime_authority: (*runtime.runtime_for_test()).authority_identity().as_u64(),
            schema: runtime.installed_schema().binding_identity(),
            scope: WorthQueryOperationScopeEntityBinding::from_entity_for_test(anchor),
            output_binding: lineage.binding_identity_for_test(worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId::of_for_test::<RegionOutputBinding<OwnerSchema>>()).unwrap(),
        };
        let coordinate = product.observation_for_test();
        let cells = &(*lineage.by_source_for_test())[&source][&coordinate.lifecycle_incarnation()][&coordinate.reference_generation().get()];
        assert_eq!(cells.len(), 1, "exact initial performed generation has one publication");
        let row = cells[0].get().unwrap();
        assert!((*row.performed_origin_for_test()).is_none());
        // This locator is installed only by the required-claims commit handoff.
        // It proves that the Query completion reached required_origin::bind;
        // merely lacking native custody would be a wrong-reason negative.
        let locator = (*row.native_prior_checkpoint_for_test()).as_ref().expect("real Query required-output publication handoff ran");
        let producer = locator.producer_for_test().to_owned();
        let dependency = (*row.producer_dependency_identity_for_test()).unwrap();
        let meaning = RequiredOutputPublicationMeaning {
            implementation: (*row.producing_implementation_for_test()).expect("the actual producer publication retains its original implementation"),
            family: settled.output_family_identity().to_owned(), source_scope: anchor,
            source_partition: (*row.source_partition_identity_for_test()).unwrap(), source_epoch: locator.source_for_test(),
        };
        drop(lineage);
        let created: Vec<_> = roles.iter().filter(|role| role.posture == RequiredOutputRolePosture::Create).cloned().collect();
        assert_eq!(created.len(), 3);
        graph.with_runtime(&execution, |native| {
            for (label, targets) in [("whole mixed correspondence", roles.as_slice()), ("created-only ring", created.as_slice())] {
                let result = native.readmit_required_output_publication(&execution, &publisher.native, product.relational_basis_for_test(), &producer, &dependency, &meaning, targets);
                let denial = result.expect_err(label);
                assert!(matches!(denial, worth_relational::facade::transactions::TransactionCommitError::Publication { .. }), "{label}: ownership court, not request/funding refusal: {denial:?}");
            }
        });
    }).unwrap();
}
