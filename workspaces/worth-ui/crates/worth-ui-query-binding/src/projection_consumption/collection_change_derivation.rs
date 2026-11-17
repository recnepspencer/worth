use std::collections::BTreeMap;

use worth_query::facade::installed::collection::{
    WorthQueryCollectionPatchApplicationReceipt, WorthQueryCollectionPatchOperation,
    WorthQueryCollectionRowHandle,
};

use super::{
    derive_collection_projection, UiCollectionDerivationContext, UiCollectionProjectionChange,
    UiCollectionProjectionDelivery, UiCollectionProjectionFactReceipt,
    UiCollectionProjectionRowReference,
};

pub(crate) fn derive_applied_collection_projection(
    context: UiCollectionDerivationContext<'_>,
    receipt: &WorthQueryCollectionPatchApplicationReceipt,
) -> Result<
    UiCollectionProjectionFactReceipt,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let mut changed_rows = BTreeMap::new();
    let mut changes = Vec::new();
    for operation in receipt.operations() {
        translate_operation(operation, &mut changed_rows, &mut changes);
    }
    let rows = changed_rows.values().copied().collect::<Vec<_>>();
    derive_collection_projection(
        context,
        &rows,
        UiCollectionProjectionDelivery::Patch,
        changes.into_boxed_slice(),
    )
}

fn translate_operation<'a>(
    operation: &'a WorthQueryCollectionPatchOperation,
    changed_rows: &mut BTreeMap<
        worth_query::facade::foundation::WorthQueryEntityIdentity,
        &'a WorthQueryCollectionRowHandle,
    >,
    changes: &mut Vec<UiCollectionProjectionChange>,
) {
    match operation {
        WorthQueryCollectionPatchOperation::Insert { row, at } => {
            changed_rows.insert(row.entity_identity().clone(), row);
            changes.push(UiCollectionProjectionChange::Insert {
                row: row_reference(row.entity_identity()),
                at: *at,
            });
        }
        WorthQueryCollectionPatchOperation::Remove { entity, from } => {
            changes.push(UiCollectionProjectionChange::Remove {
                row: row_reference(entity),
                from: *from,
            });
        }
        WorthQueryCollectionPatchOperation::Move { row, from, to } => {
            changes.push(UiCollectionProjectionChange::Move {
                row: row_reference(row.entity_identity()),
                from: *from,
                to: *to,
            });
        }
        WorthQueryCollectionPatchOperation::Regroup { entity, from, to } => {
            changes.push(UiCollectionProjectionChange::Regroup {
                row: row_reference(entity),
                from: from.clone().map(Vec::into_boxed_slice),
                to: to.clone().map(Vec::into_boxed_slice),
            });
        }
        WorthQueryCollectionPatchOperation::Update { row } => {
            changed_rows.insert(row.entity_identity().clone(), row);
            changes.push(UiCollectionProjectionChange::Update {
                row: row_reference(row.entity_identity()),
            });
        }
        WorthQueryCollectionPatchOperation::WindowShift { .. } => {
            changes.push(UiCollectionProjectionChange::WindowShift);
        }
        WorthQueryCollectionPatchOperation::ResetRequired { reason, .. } => {
            changes.push(UiCollectionProjectionChange::ResetRequired {
                reason: crate::collection_delivery::map_reset_reason(*reason),
            });
        }
        WorthQueryCollectionPatchOperation::ResultState { .. }
        | WorthQueryCollectionPatchOperation::Warnings { .. }
        | WorthQueryCollectionPatchOperation::Continuation { .. } => {}
    }
}

fn row_reference(
    identity: &worth_query::facade::foundation::WorthQueryEntityIdentity,
) -> UiCollectionProjectionRowReference {
    UiCollectionProjectionRowReference::query_issued(identity.evidence_identity())
}

#[test]
fn regroup_preserves_query_row_identity_and_exact_group_paths() {
    let mut workspace = crate::scalar_text_projection_fixture::collection_projection_workspace();
    let entity = crate::scalar_text_projection_fixture::insert_collection_status(
        &mut workspace,
        "pulse.alpha",
        "Alpha",
    );
    let operation = WorthQueryCollectionPatchOperation::Regroup {
        entity: entity.clone(),
        from: Some(vec!["prior".to_owned()]),
        to: Some(vec!["successor".to_owned(), "nested".to_owned()]),
    };
    let mut changed_rows = BTreeMap::new();
    let mut changes = Vec::new();

    translate_operation(&operation, &mut changed_rows, &mut changes);

    assert!(changed_rows.is_empty());
    assert!(matches!(
        changes.as_slice(),
        [UiCollectionProjectionChange::Regroup { row, from, to }]
            if row.query_identity() == &entity.evidence_identity()
                && from.as_deref() == Some(["prior".to_owned()].as_slice())
                && to.as_deref()
                    == Some(["successor".to_owned(), "nested".to_owned()].as_slice())
    ));
}
