//! Malformed forests cannot become installed Source programs.
use super::*;
#[test]
fn nested_projection_binding_rejects_parent_cycles_kind_gaps_and_foreign_routes() {
    let (registry, program) = registry::profile(registry::Profile::NestedFields);
    let identity = |name: &str| DerivedRelationIdentity {
        schema: SchemaId("managed-source-court".into()),
        version: SchemaVersionId(1),
        name: name.into(),
    };
    let root = registry
        .derived_relation(&identity("directory-items"))
        .unwrap();
    let observed = registry
        .derived_relation(&identity("directory-observes"))
        .unwrap();
    let reverse = registry.derived_relation(&identity("item-owner")).unwrap();
    for parent in [0, 1, usize::MAX] {
        let routes = vec![ManagedIndexSourceProjectionRoute::nested(
            "bad-parent".into(),
            parent,
            observed.clone(),
            vec![],
        )
        .unwrap()];
        assert!(matches!(
            registry.bind_managed_index_source(&root, vec![], routes),
            Err(DerivedRelationDenial::InvalidExpression)
        ));
    }
    let bad_kind = vec![
        program.projection_routes()[0].clone(),
        ManagedIndexSourceProjectionRoute::nested("bad-kind".into(), 0, reverse, vec![]).unwrap(),
    ];
    assert!(matches!(
        registry.bind_managed_index_source(&root, vec![], bad_kind),
        Err(DerivedRelationDenial::InvalidExpression)
    ));
    let (foreign, _program) = registry::profile(registry::Profile::NestedFields);
    let routes = vec![
        program.projection_routes()[0].clone(),
        ManagedIndexSourceProjectionRoute::nested(
            "foreign".into(),
            0,
            foreign
                .derived_relation(&identity("directory-observes"))
                .unwrap(),
            vec![],
        )
        .unwrap(),
    ];
    assert!(matches!(
        registry.bind_managed_index_source(&root, vec![], routes),
        Err(DerivedRelationDenial::InvalidExpression)
    ));
}
