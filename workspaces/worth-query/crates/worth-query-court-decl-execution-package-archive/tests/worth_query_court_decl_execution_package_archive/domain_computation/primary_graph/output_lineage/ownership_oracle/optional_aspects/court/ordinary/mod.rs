//! Ordinary content is genuine; producer ownership is a distinct native court.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::*;

pub fn assert_ordinary<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    managed: &WorthQueryOutputDemandSettlement,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    readmit_with_ordinary(runtime, managed, Some(receipt));
}
pub fn roles_and_nonownership<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Vec<RequiredOutputRoleTarget> {
    let graph = &(*(*runtime.primary_provider_for_test()).graph_for_test());
    let correspondence = receipt.retain_output_correspondence_for_test();
    let lineage = graph.output_lineage.lock().unwrap();
    let rows: Vec<_> = (*lineage
        .by_source_for_test())
        .values()
        .flat_map(|l| l.values())
        .flat_map(|g| g.values())
        .flat_map(|rows| rows.iter())
        .filter_map(|row| row.get())
        .filter(|row| std::sync::Arc::ptr_eq(&(*row.correspondence_for_test()), &correspondence))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "the actual ordinary receipt has one performed record"
    );
    let row = rows[0];
    assert!(
        (*row.native_output_witness_for_test())
            .get()
            .and_then(|cell| cell.get())
            .is_some(),
        "ordinary native content has a content witness, not producer authority"
    );
    assert!((*row.required_publication_for_test()).is_none());
    assert!((*row.native_prior_checkpoint_for_test()).is_none());
    assert!((*row.producer_dependency_identity_for_test()).is_none());
    let roles: Vec<_> = correspondence
        .native_witness_roles_for_test()
        .map(|(role, posture, name, id)| {
            assert_eq!(posture, WorthQueryApplicationOutputPosture::Create);
            RequiredOutputRoleTarget {
                role: role.to_owned(),
                entity_name: name.to_owned(),
                kind: (*graph.layout_for_test()).entity_kind_for_test(name).unwrap(),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Existing(id),
            }
        })
        .collect();
    assert_eq!(roles.len(), 4);
    roles
}
pub fn observe<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    roles: &[RequiredOutputRoleTarget],
) {
    let primary = receipt
        .output_correspondence()
        .entity_for_binding_role_for_test::<OrdinaryBinding<Schema>>(
            <BornPresent<Schema> as WorthQueryApplicationOutputRole>::NAME,
        )
        .unwrap();
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::publication_state::observe_roles(runtime, phase, primary, roles);
}
