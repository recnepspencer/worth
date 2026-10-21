//! Observe the original committed roster even when its read makes it born stale.
use super::*;
pub(in super::super) fn observe_performed_receipt<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    posture: WorthQueryApplicationOutputPosture,
) -> (Vec<EntityId>, RelationId) {
    let roles: Vec<_> = receipt
        .output_correspondence()
        .native_witness_roles()
        .map(|(role, actual, name, entity)| {
            assert_eq!(actual, posture);
            RequiredOutputRoleTarget {
                role: role.to_owned(),
                entity_name: name.to_owned(),
                kind: runtime
                    .primary_provider
                    .graph
                    .layout
                    .entity_kind(name)
                    .unwrap(),
                posture: match posture {
                    WorthQueryApplicationOutputPosture::Create => RequiredOutputRolePosture::Create,
                    WorthQueryApplicationOutputPosture::Preserve => {
                        RequiredOutputRolePosture::Preserve
                    }
                    _ => panic!("only performed birth or Preserve"),
                },
                target: EntityReference::Existing(entity),
            }
        })
        .collect();
    assert_eq!(roles.len(), 4);
    assert!(
        receipt.exact_output_settlement().is_some(),
        "the receipt is the exact recorded original"
    );
    let primary = if posture == WorthQueryApplicationOutputPosture::Create {
        receipt.output_correspondence().entity_for_binding_role::<Binding<Schema, false>>(
            <<Initial<Schema> as WorthQueryApplicationProducerBinding<Schema>>::OutputRole as WorthQueryApplicationOutputRole>::NAME)
    } else {
        receipt.output_correspondence().entity_for_binding_role::<Binding<Schema, true>>(
            <<Preserve<Schema> as WorthQueryApplicationProducerBinding<Schema>>::OutputRole as WorthQueryApplicationOutputRole>::NAME)
    }.unwrap();
    let incidence = publication_state::observe_roles(runtime, phase, primary, &roles);
    let ids = roles
        .into_iter()
        .map(|role| {
            let EntityReference::Existing(id) = role.target else {
                unreachable!()
            };
            id
        })
        .collect();
    (ids, incidence)
}
