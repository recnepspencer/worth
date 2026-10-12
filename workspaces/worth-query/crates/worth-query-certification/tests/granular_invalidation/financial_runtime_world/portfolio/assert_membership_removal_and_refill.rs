//! assert membership removal and refill.
use super::*;

pub(super) fn assert_membership_removal_and_refill(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let primary = row_identities(query)[0].clone();
    host.amend_portfolio_desk(4, "credit");
    host.portfolio_clock_control.push(4, 13);
    let performed = perform(
        request_execution,
        "membership-removal",
        host,
        query,
        binding,
    );
    assert_roles(
        &performed,
        &[
            Role::ProjectedValue,
            Role::SelectionOrMembership,
            Role::Grouping,
        ],
    );
    let patch = indexed(&performed);
    let rates_group = vec![
        worth_foundational::facade::prepare_aspect_value_identity_basis(
            &worth_foundational::facade::AspectValue::String("rates".into()),
        )
        .as_str()
        .to_owned(),
    ];
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        WorthQueryCollectionPatchOperation::Remove { entity, .. } if entity == &primary
    )));
    assert!(
        patch.operations().iter().any(|operation| matches!(
            operation,
            WorthQueryCollectionPatchOperation::Regroup { entity, from: Some(from), to: None }
                if entity == &primary && from == &rates_group
        )),
        "missing group transition in {:?}",
        patch.operations()
    );
    assert!(!row_identities(query).contains(&primary));
    assert_eq!(
        patch.rows(),
        query.collection.as_ref().unwrap().current_rows()
    );
    assert_matches_fresh(request_execution, host, query);
}
