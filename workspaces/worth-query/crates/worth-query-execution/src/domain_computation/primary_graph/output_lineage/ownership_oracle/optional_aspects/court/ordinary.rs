//! Ordinary content is genuine; producer ownership is a distinct native court.
use super::*;

pub(in crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects) fn assert_ordinary<
    Schema: BirthSchema,
>(
    runtime: &installation::Application<Schema>,
    managed: &WorthQueryOutputDemandSettlement,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    readmit_with_ordinary(runtime, managed, Some(receipt));
}
pub(super) fn roles_and_nonownership<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Vec<RequiredOutputRoleTarget> {
    let graph = &runtime.primary_provider.graph;
    let correspondence = receipt.retain_output_correspondence();
    let lineage = graph.output_lineage.lock().unwrap();
    let rows: Vec<_> = lineage
        .by_source
        .values()
        .flat_map(|l| l.values())
        .flat_map(|g| g.values())
        .flat_map(|rows| rows.iter())
        .filter_map(|row| row.get())
        .filter(|row| std::sync::Arc::ptr_eq(&row.correspondence, &correspondence))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "the actual ordinary receipt has one performed record"
    );
    let row = rows[0];
    assert!(
        row.native_output_witness
            .get()
            .and_then(|cell| cell.get())
            .is_some(),
        "ordinary native content has a content witness, not producer authority"
    );
    assert!(row.required_publication.is_none());
    assert!(row.native_prior_checkpoint.is_none());
    assert!(row.producer_dependency_identity.is_none());
    let roles: Vec<_> = correspondence
        .native_witness_roles()
        .map(|(role, posture, name, id)| {
            assert_eq!(posture, WorthQueryApplicationOutputPosture::Create);
            RequiredOutputRoleTarget {
                role: role.to_owned(),
                entity_name: name.to_owned(),
                kind: graph.layout.entity_kind(name).unwrap(),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Existing(id),
            }
        })
        .collect();
    assert_eq!(roles.len(), 4);
    roles
}
pub(super) fn observe<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    roles: &[RequiredOutputRoleTarget],
) {
    let primary = receipt
        .output_correspondence()
        .entity_for_binding_role::<OrdinaryBinding<Schema>>(
            <BornPresent<Schema> as WorthQueryApplicationOutputRole>::NAME,
        )
        .unwrap();
    publication_state::observe_roles(runtime, phase, primary, roles);
}
