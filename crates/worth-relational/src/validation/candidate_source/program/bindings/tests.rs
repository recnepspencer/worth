//! Declared names locate a recipe; only its registry's live binding is reused.
use super::*;
use crate::tests::support::managed_cache_schema;
#[test]
fn managed_cache_binding_interns_exact_recipe_and_refuses_a_conflicting_live_recipe() {
    let (registry, program) = managed_cache_schema();
    let path = program.0.path.clone();
    let same = registry
        .clone()
        .bind_managed_index_source(&path, vec![], vec![])
        .unwrap();
    assert_eq!(program, same);
    let route =
        ManagedIndexSourceProjectionRoute::new("endpoint".into(), path.clone(), vec![]).unwrap();
    let conflict = registry.bind_managed_index_source(&path, vec![], vec![route]);
    assert!(
        matches!(&conflict, Err(Denial::DuplicateDefinition(identity)) if identity == &path.parts().identity),
        "{conflict:?}"
    );
    let (_, foreign) = managed_cache_schema();
    assert_eq!(foreign.0.path.parts().identity, path.parts().identity);
    assert_ne!(
        foreign, program,
        "equal descriptive identities cannot share live authority"
    );
}

#[test]
fn managed_cache_binding_follows_actual_path_owner_across_independent_schema_extensions() {
    let (base, original) = managed_cache_schema();
    let mut definition = original.0.path.parts().clone();
    definition.identity.name = "managed-index-source::independent-extension".into();
    let identity = definition.identity.clone();
    let first = base
        .clone()
        .register_derived_relations(vec![definition.clone()])
        .unwrap();
    let second = base.register_derived_relations(vec![definition]).unwrap();
    let a = first
        .bind_managed_index_source(&first.derived_relation(&identity).unwrap(), vec![], vec![])
        .unwrap();
    let b = second
        .bind_managed_index_source(&second.derived_relation(&identity).unwrap(), vec![], vec![])
        .unwrap();
    assert_ne!(
        a, b,
        "independent actual registrations remain different live bindings"
    );
    assert_eq!(a.0.path.parts().identity, b.0.path.parts().identity);
    assert_eq!(
        first
            .bind_managed_index_source(&original.0.path, vec![], vec![])
            .unwrap(),
        original,
        "extending a registry preserves an existing path owner's live binding"
    );
}

#[test]
fn public_source_registration_rejects_a_foreign_route_even_with_its_actual_membership_path() {
    let (base, original) = managed_cache_schema();
    let membership = original.0.path.clone();
    let mut route = membership.parts().clone();
    route.identity.name = "independent-projection-route".into();
    let identity = route.identity.clone();
    let source = base
        .clone()
        .register_derived_relations(vec![route.clone()])
        .unwrap();
    let receiving = base.register_derived_relations(vec![route]).unwrap();
    assert!(receiving.admits_derived_relation(&membership));
    drop(original);
    let program = source
        .bind_managed_index_source(
            &membership,
            vec![],
            vec![ManagedIndexSourceProjectionRoute::new(
                "endpoint".into(),
                source.derived_relation(&identity).unwrap(),
                vec![],
            )
            .unwrap()],
        )
        .unwrap();
    assert_eq!(
        program
            .require_registry(
                &source,
                crate::schema::SchemaCompilationControl::Configuration
            )
            .map_err(crate::validation::CustomInvariantCompilationDenial::configuration_source),
        Ok(())
    );
    assert_eq!(
        program
            .require_registry(
                &receiving,
                crate::schema::SchemaCompilationControl::Configuration
            )
            .map_err(crate::validation::CustomInvariantCompilationDenial::configuration_source),
        Err(Denial::ForeignDefinition)
    );
}
