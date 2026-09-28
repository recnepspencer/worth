use worth_foundational::facade::{AspectValue, InternedString};

pub(in crate::domain_computation::primary_graph::application_attempt) fn transition_entity_in_receipt(
    receipt: &crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt,
    transition_identity: &str,
    transition_identity_locator: &worth_foundational::facade::AspectFieldLocator,
) -> Option<worth_relational::facade::identity::EntityId> {
    let expected = AspectValue::String(InternedString::Raw(transition_identity.to_owned()));
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            (receipt
                .committed_changes()
                .committed_field_values(entity, &[transition_identity_locator])
                == Some(vec![expected.clone()]))
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    let [transition] = candidates.as_slice() else {
        return None;
    };
    Some(*transition)
}
