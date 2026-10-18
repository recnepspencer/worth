//! Active native court, using actual installed publisher and exact performed row.
use super::installation::{Application, OwnerSchema};
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use worth_relational::facade::{
    mvcc::{RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget},
    transactions::EntityReference,
};

pub(super) fn refuse_mixed_and_created_subset(
    runtime: &Application,
    settled: &WorthQueryOutputDemandSettlement,
) {
    let scope = primary_graph::tests::fixture::live_scope();
    runtime.with_application_advancement(&scope, |phase| {
        let execution = phase.execution_request_for(&runtime.product_runtime).unwrap();
        let (_, product, _application_basis) = runtime.on_branch(runtime.current_world()).select(&phase).unwrap().into_parts();
        let graph = &runtime.primary_provider.graph;
        let publisher = graph.application_publication.as_ref().expect("actual installed publication owner");
        let roles: Vec<_> = settled.output_correspondence.native_witness_roles().map(|(role, posture, name, entity)| RequiredOutputRoleTarget {
            role: role.to_owned(), entity_name: name.to_owned(), kind: graph.layout.entity_kind(name).unwrap(),
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
        let source = super::super::SemanticSource {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema: runtime.installed_schema().binding_identity(),
            scope: WorthQueryOperationScopeEntityBinding::from_entity(anchor),
            output_binding: lineage.binding_identity(crate::equality_type_id::EqualityTypeId::of::<RegionOutputBinding<OwnerSchema>>()).unwrap(),
        };
        let coordinate = product.observation();
        let cells = &lineage.by_source[&source][&coordinate.lifecycle_incarnation()][&coordinate.reference_generation().get()];
        assert_eq!(cells.len(), 1, "exact initial performed generation has one publication");
        let row = cells[0].get().unwrap();
        assert!(row.performed_origin.is_none());
        // This locator is installed only by the required-claims commit handoff.
        // It proves that the Query completion reached required_origin::bind;
        // merely lacking native custody would be a wrong-reason negative.
        let locator = row.native_prior_checkpoint.as_ref().expect("real Query required-output publication handoff ran");
        let producer = locator.producer().to_owned();
        let dependency = row.producer_dependency_identity.unwrap();
        let meaning = RequiredOutputPublicationMeaning {
            family: settled.output_family_identity().to_owned(), source_scope: anchor,
            source_partition: row.source_partition_identity.unwrap(), source_epoch: locator.source(),
        };
        drop(lineage);
        let created: Vec<_> = roles.iter().filter(|role| role.posture == RequiredOutputRolePosture::Create).cloned().collect();
        assert_eq!(created.len(), 3);
        graph.with_runtime(&execution, |native| {
            for (label, targets) in [("whole mixed correspondence", roles.as_slice()), ("created-only ring", created.as_slice())] {
                let result = native.readmit_required_output_publication(&execution, &publisher.native, product.relational_basis(), &producer, &dependency, &meaning, targets);
                let denial = result.expect_err(label);
                assert!(matches!(denial, worth_relational::facade::transactions::TransactionCommitError::Publication { .. }), "{label}: ownership court, not request/funding refusal: {denial:?}");
            }
        });
    }).unwrap();
}
