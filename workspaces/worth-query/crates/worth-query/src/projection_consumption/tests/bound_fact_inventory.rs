use super::phase_three_contracts::{admitted, query_read_source, test_binding};
use crate::projection_consumption::{ProjectMaterializedFacts, ProjectionFactKind};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

#[test]
fn bound_fact_inventory_preserves_requested_kind_and_field_shape() {
    let field_path = || {
        crate::projection_consumption::projection_fact_field_path_from_segments([
            FieldKey::new("profile").expect("profile field should admit"),
            FieldKey::new("display_name").expect("display-name field should admit"),
        ])
    };
    let contract = admitted(
        query_read_source(),
        test_binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(field_path())
            .derived_field_path(field_path()),
    )
    .bind_contract()
    .expect("fixture digest must fit");

    let kinds = contract
        .fact_families()
        .iter()
        .map(|fact| {
            (
                fact.kind(),
                fact.field_path()
                    .and_then(|field_path| field_path.canonical_field_path().cloned()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            (ProjectionFactKind::EntityIdentity, None),
            (
                ProjectionFactKind::DisplayField,
                Some(canonical_field_path("profile.display_name")),
            ),
            (
                ProjectionFactKind::DerivedField,
                Some(canonical_field_path("profile.display_name")),
            ),
        ]
    );
}

fn canonical_field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.')
            .map(|segment| FieldKey::new(segment.to_string()))
            .collect::<Option<Vec<_>>>()
            .expect("test field path should be canonical"),
    )
    .expect("test field path should not be empty")
}
