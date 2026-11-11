use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::{
    AspectFieldKey, NativeComparisonOperator, RelationName, WorthQueryPredicateOperand,
};
use crate::declarative_live::{
    DeclarativeLiveQueryRequest, DeclarativeOrderingField, DeclarativePredicateFilter,
};
use crate::memory_workspace::WorthQueryEntity;
use crate::runtime::{WorthQueryReadBuiltInOperator, WorthQueryReadGraph};
use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
};

mod ordering_key;

pub(crate) use ordering_key::{canonical_ordering_key, WorthQueryCanonicalOrderingKey};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WorthQueryReadMaterializedRowIdentity {
    value: String,
}

impl WorthQueryReadMaterializedRowIdentity {
    fn from_label(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

pub(super) fn materialize_detail_rows_from_request(
    read_graph: &WorthQueryReadGraph,
    request: &DeclarativeLiveQueryRequest,
    rows: &[WorthQueryEntity],
) -> Option<Vec<WorthQueryEntity>> {
    let Some(anchor) = identity_anchor(request) else {
        return (rows.len() == 1).then(|| order_rows(vec![rows[0].clone()], request));
    };
    let anchor_identity = WorthQueryReadMaterializedRowIdentity::from_label(anchor);
    let row_index = row_index(rows);
    let anchor_row = row_index.get(&anchor_identity)?.clone();
    let selected = if uses_shared_neighborhood(read_graph) {
        collect_shared_neighborhood_rows(&anchor_row, rows, request)?
    } else if request.traversal().is_empty() {
        vec![anchor_row]
    } else {
        collect_traversal_rows(&anchor_row, &row_index, request)?
    };
    Some(order_rows(selected, request))
}

pub(super) fn materialize_collection_rows_from_request(
    read_graph: &WorthQueryReadGraph,
    request: &DeclarativeLiveQueryRequest,
    rows: &[WorthQueryEntity],
) -> Vec<WorthQueryEntity> {
    let roots = rows
        .iter()
        .filter(|row| row_matches_predicates(row, request.predicate_filters()))
        .cloned()
        .collect::<Vec<_>>();
    if request.traversal().is_empty() {
        return order_rows(roots, request);
    }

    let row_index = row_index(rows);
    let mut selected = BTreeMap::new();
    for root in roots {
        let neighborhood = if uses_shared_neighborhood(read_graph) {
            collect_shared_neighborhood_rows(&root, rows, request)
        } else {
            collect_traversal_rows(&root, &row_index, request)
        };
        for row in neighborhood.unwrap_or_else(|| vec![root]) {
            selected.insert(row.identity().terminal_projection_for_reporting(), row);
        }
    }
    order_rows(selected.into_values().collect(), request)
}

pub(super) fn synthetic_detail_rows_for_request(
    request: &DeclarativeLiveQueryRequest,
) -> Result<
    Vec<WorthQueryEntity>,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let anchor_identity = identity_anchor(request).unwrap_or("synthetic-anchor");
    Ok(vec![WorthQueryEntity::from_aspect_projection(
        crate::memory_workspace::admit_authored_entity_label(anchor_identity)?,
        BTreeMap::from([(
            worth_foundational::facade::AspectKey::new("read.synthetic").unwrap(),
            AspectValue::Bool(true),
        )]),
        BTreeMap::from([(
            worth_foundational::facade::AspectKey::new("identity").unwrap(),
            worth_foundational::facade::StructAspectValue::new([(
                FieldKey::new("id").unwrap(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    anchor_identity.to_string(),
                ),
            )])
            .unwrap(),
        )]),
        BTreeMap::new(),
    )])
}

pub fn worth_query_materialized_relation_field_key(relation: &RelationName) -> FieldKey {
    let encoded = relation
        .as_str()
        .chars()
        .map(materialized_relation_slot_fragment)
        .collect::<String>();
    FieldKey::new(encoded).expect("encoded relation slot must be a foundational field key")
}

fn uses_shared_neighborhood(read_graph: &WorthQueryReadGraph) -> bool {
    read_graph
        .built_in_operators()
        .contains(&WorthQueryReadBuiltInOperator::SharedEndpoint)
        || read_graph
            .built_in_operators()
            .contains(&WorthQueryReadBuiltInOperator::SharedAttachment)
}

fn collect_shared_neighborhood_rows(
    anchor_row: &WorthQueryEntity,
    rows: &[WorthQueryEntity],
    request: &DeclarativeLiveQueryRequest,
) -> Option<Vec<WorthQueryEntity>> {
    let anchor_identity = row_identity_label(anchor_row)?;
    let anchor_targets = request
        .traversal()
        .iter()
        .filter_map(|selector| relation_target(anchor_row, selector.relation_name()))
        .collect::<BTreeSet<_>>();
    Some(
        rows.iter()
            .filter(|row| {
                row_identity_label(row)
                    .as_ref()
                    .is_some_and(|identity| identity == &anchor_identity)
                    || request.traversal().iter().any(|selector| {
                        relation_target(row, selector.relation_name())
                            .is_some_and(|target| anchor_targets.contains(&target))
                    })
            })
            .cloned()
            .collect(),
    )
}

fn collect_traversal_rows(
    anchor_row: &WorthQueryEntity,
    row_index: &BTreeMap<WorthQueryReadMaterializedRowIdentity, WorthQueryEntity>,
    request: &DeclarativeLiveQueryRequest,
) -> Option<Vec<WorthQueryEntity>> {
    let anchor_identity = row_identity_label(anchor_row)?;
    let mut selected = BTreeMap::from([(anchor_identity.clone(), anchor_row.clone())]);
    for selector in request.traversal() {
        let mut current = anchor_identity.clone();
        for _ in 0..usize::from(selector.depth()) {
            let Some(next_identity) = row_index
                .get(&current)
                .and_then(|row| relation_target(row, selector.relation_name()))
            else {
                break;
            };
            let Some(next_row) = row_index.get(&next_identity) else {
                break;
            };
            let Some(next_identity_label) = row_identity_label(next_row) else {
                break;
            };
            selected.insert(next_identity_label, next_row.clone());
            current = next_identity;
        }
    }
    Some(selected.into_values().collect())
}

fn order_rows(
    mut rows: Vec<WorthQueryEntity>,
    request: &DeclarativeLiveQueryRequest,
) -> Vec<WorthQueryEntity> {
    rows.sort_by(|left, right| compare_ordered_rows(left, right, request.ordering()));
    rows
}

pub(crate) fn compare_ordered_rows(
    left: &WorthQueryEntity,
    right: &WorthQueryEntity,
    ordering: &[DeclarativeOrderingField],
) -> Ordering {
    ordering
        .iter()
        .map(|entry| compare_ordering_entry(left, right, entry))
        .find(|ordering| !ordering.is_eq())
        .unwrap_or_else(|| left.identity().cmp(right.identity()))
}

fn compare_ordering_entry(
    left: &WorthQueryEntity,
    right: &WorthQueryEntity,
    ordering: &DeclarativeOrderingField,
) -> Ordering {
    let comparison = row_field_value(left, ordering.source_field_key())
        .cmp(&row_field_value(right, ordering.source_field_key()));
    match ordering.direction() {
        crate::authoring::OrderingDirection::Ascending => comparison,
        crate::authoring::OrderingDirection::Descending => comparison.reverse(),
    }
}

fn identity_anchor(request: &DeclarativeLiveQueryRequest) -> Option<&str> {
    request
        .predicate_filters()
        .iter()
        .find_map(|predicate| match predicate {
            DeclarativePredicateFilter::Equality(filter)
                if is_identity_field_key(filter.source_field_key()) =>
            {
                match filter.value().as_native() {
                    AspectValue::String(InternedString::Stored(value)) => {
                        let value = value.as_string();
                        Some(value.as_str())
                    }
                    AspectValue::String(InternedString::Raw(value)) => Some(value.as_str()),
                    _ => None,
                }
            }
            _ => None,
        })
}

fn relation_target(
    row: &WorthQueryEntity,
    relation: &RelationName,
) -> Option<WorthQueryReadMaterializedRowIdentity> {
    row.scalar_value_at(&relation_target_field_path(relation))
        .and_then(as_string_scalar)
        .map(WorthQueryReadMaterializedRowIdentity::from_label)
}

fn relation_target_field_path(relation: &RelationName) -> CanonicalFieldPath {
    CanonicalFieldPath::new([
        FieldKey::new("relations").expect("relations slot must be foundational"),
        worth_query_materialized_relation_field_key(relation),
    ])
    .expect("relation target field path must not be empty")
}

fn materialized_relation_slot_fragment(value: char) -> String {
    match value {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => value.to_string(),
        '.' => "_dot_".to_string(),
        '-' => "_dash_".to_string(),
        ':' => "_colon_".to_string(),
        '/' => "_slash_".to_string(),
        '\\' => "_backslash_".to_string(),
        value if value.is_whitespace() => "_space_".to_string(),
        value => format!("_u{:x}_", value as u32),
    }
}

fn row_index(
    rows: &[WorthQueryEntity],
) -> BTreeMap<WorthQueryReadMaterializedRowIdentity, WorthQueryEntity> {
    rows.iter()
        .cloned()
        .filter_map(|row| row_identity_label(&row).map(|identity| (identity, row)))
        .collect()
}

fn row_identity_label(row: &WorthQueryEntity) -> Option<WorthQueryReadMaterializedRowIdentity> {
    row.scalar_value_at(&native_field_path("identity.id"))
        .and_then(as_string_scalar)
        .map(WorthQueryReadMaterializedRowIdentity::from_label)
}

pub(crate) fn row_matches_predicates(
    row: &WorthQueryEntity,
    predicates: &[DeclarativePredicateFilter],
) -> bool {
    predicates
        .iter()
        .all(|predicate| row_matches_predicate(row, predicate))
}

fn row_matches_predicate(row: &WorthQueryEntity, predicate: &DeclarativePredicateFilter) -> bool {
    let value = row_field_value(row, predicate.source_field_key());
    match predicate {
        DeclarativePredicateFilter::Equality(filter) => {
            value.is_some_and(|value| aspect_value_matches_scalar(value, filter.value()))
        }
        DeclarativePredicateFilter::NativeComparison(filter) => value.is_some_and(|value| {
            let expected = filter.value().as_native();
            value.value_family() == expected.value_family()
                && match filter.operator() {
                    NativeComparisonOperator::GreaterThan => value > expected,
                    NativeComparisonOperator::LessThan => value < expected,
                }
        }),
        DeclarativePredicateFilter::StringContains(filter) => value
            .and_then(as_string_scalar)
            .is_some_and(|value| value.contains(filter.value())),
        DeclarativePredicateFilter::SetMembership(filter) => value.is_some_and(|value| {
            filter
                .values()
                .iter()
                .any(|candidate| aspect_value_matches_scalar(value, candidate))
        }),
        DeclarativePredicateFilter::Presence(_) => value.is_some(),
    }
}

fn predicate_field_path(field: &AspectFieldKey) -> CanonicalFieldPath {
    native_field_path(format!(
        "{}.{}",
        field.native_aspect_key().as_str(),
        field.native_field_key().as_str()
    ))
}

fn row_field_value<'a>(
    row: &'a WorthQueryEntity,
    field: &AspectFieldKey,
) -> Option<&'a AspectValue> {
    row.struct_aspect_value(&field.native_aspect_key())
        .and_then(|value| value.get(&field.native_field_key()))
        .or_else(|| row.aspect_value(&field.native_aspect_key()))
        .or_else(|| row.scalar_value_at(&predicate_field_path(field)))
}

fn aspect_value_matches_scalar(value: &AspectValue, expected: &WorthQueryPredicateOperand) -> bool {
    value == expected.as_native()
}

fn is_identity_field_key(field: &AspectFieldKey) -> bool {
    field.native_aspect_key() == identity_aspect_key()
        && field.native_field_key() == identity_field_key()
}

fn identity_aspect_key() -> AspectKey {
    AspectKey::new("identity").expect("identity aspect key should be foundational")
}

fn identity_field_key() -> FieldKey {
    FieldKey::new("id").expect("identity field key should be foundational")
}

fn as_string_scalar(value: &AspectValue) -> Option<&str> {
    match value {
        AspectValue::String(InternedString::Stored(value)) => {
            let value = value.as_string();
            Some(value.as_str())
        }
        AspectValue::String(InternedString::Raw(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn native_field_path(path: impl AsRef<str>) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.as_ref()
            .split('.')
            .map(FieldKey::new)
            .collect::<Option<Vec<_>>>()
            .expect("materialized row field path segments must be foundational"),
    )
    .expect("materialized row field path must not be empty")
}
