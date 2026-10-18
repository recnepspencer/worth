use std::collections::BTreeSet;

use super::*;

pub(super) fn validate_collection(
    contract: &WorthQueryOperationCollectionContract,
    canonical_query: &worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
) -> Result<(), &'static str> {
    let WorthQueryOperationCollectionContract::Collection {
        row_identity_field,
        ordering_fields,
        grouping,
        window,
        continuation,
        ..
    } = contract
    else {
        return Ok(());
    };
    if ordering_fields.is_empty() {
        return Err("empty-collection-ordering");
    }
    if ordering_fields.iter().collect::<BTreeSet<_>>().len() != ordering_fields.len() {
        return Err("duplicate-ordering-field");
    }
    validate_collection_query_contract(
        row_identity_field,
        ordering_fields,
        grouping,
        canonical_query,
    )?;
    validate_grouping(grouping)?;
    validate_window_continuation(window, *continuation)
}

fn validate_grouping(grouping: &WorthQueryOperationGroupingContract) -> Result<(), &'static str> {
    let WorthQueryOperationGroupingContract::Grouped { grouping_fields } = grouping else {
        return Ok(());
    };
    if grouping_fields.is_empty() {
        return Err("empty-collection-grouping");
    }
    if grouping_fields.iter().collect::<BTreeSet<_>>().len() != grouping_fields.len() {
        return Err("duplicate-grouping-field");
    }
    Ok(())
}

fn validate_window_continuation(
    window: &WorthQueryOperationWindowPolicy,
    continuation: WorthQueryOperationContinuationPosture,
) -> Result<(), &'static str> {
    match (continuation, window) {
        (
            WorthQueryOperationContinuationPosture::NotRequired,
            WorthQueryOperationWindowPolicy::CompleteCollection,
        )
        | (
            WorthQueryOperationContinuationPosture::SnapshotCursor
            | WorthQueryOperationContinuationPosture::LiveCursor,
            WorthQueryOperationWindowPolicy::ContinuationBounded,
        ) => Ok(()),
        _ => Err("collection-window-continuation-mismatch"),
    }
}

fn validate_collection_query_contract(
    row_identity_field: &WorthQueryOperationCollectionField,
    ordering_fields: &[WorthQueryOperationCollectionField],
    grouping: &WorthQueryOperationGroupingContract,
    canonical_query: &worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
) -> Result<(), &'static str> {
    use worth_query_declaration::facade::authoring::QueryFamily;

    let query = canonical_query.query();
    if query.family() != &QueryFamily::Collection {
        return Err("collection-contract-requires-collection-query");
    }
    if !query
        .projection()
        .iter()
        .any(|entry| collection_field_matches(row_identity_field, entry.field_key()))
    {
        return Err("collection-row-identity-not-projected");
    }
    if ordering_fields.len() != query.ordering().len()
        || ordering_fields
            .iter()
            .zip(query.ordering())
            .any(|(declared, canonical)| !collection_field_matches(declared, canonical.field_key()))
    {
        return Err("collection-ordering-canonical-query-mismatch");
    }
    if let WorthQueryOperationGroupingContract::Grouped { grouping_fields } = grouping
        && grouping_fields.iter().any(|grouping_field| {
            !query
                .projection()
                .iter()
                .any(|entry| collection_field_matches(grouping_field, entry.field_key()))
        })
    {
        return Err("collection-grouping-field-not-projected");
    }
    Ok(())
}

fn collection_field_matches(
    collection_field: &WorthQueryOperationCollectionField,
    query_field: &worth_query_declaration::facade::authoring::AspectFieldKey,
) -> bool {
    let fields = collection_field.field_path().fields();
    fields.len() == 1
        && collection_field.aspect_key() == &query_field.native_aspect_key()
        && fields.first() == Some(&query_field.native_field_key())
}
