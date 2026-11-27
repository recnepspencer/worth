//! Installed persistent-name schemes compose through registry extension: the
//! Query installation lane lowers its kinds and schemes as additions, so
//! composition must carry them, never replace an installed meaning, and leave
//! schemaless compositions without naming state.
use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

use crate::facade::identity::KindId;
use crate::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, PersistentNameSubjectKind,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::tests::support::test_schema_registry;

fn entity_scheme(kind_name: &str, roles: [&str; 1]) -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                "worth.compose.entity.v1",
                PersistentNameSubjectKind::Entity,
                kind_name,
                roles,
                false,
                true,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    registry
}

fn named_additions() -> RelationalSchemaRegistry {
    let base = test_schema_registry();
    let kind_name = (*base.owner_entity_kinds())[&KindId(1)].kind_name.clone();
    base.install_persistent_name_schemes(
        entity_scheme(&kind_name, ["authored"]),
        BTreeMap::from([(
            (PersistentNameSubjectKind::Entity, KindId(1)),
            "worth.compose.entity.v1".to_owned(),
        )]),
    )
    .unwrap()
}

#[test]
fn extension_carries_installed_schemes_and_their_kind_bindings() {
    let composed = RelationalSchemaRegistry::new()
        .extend(named_additions())
        .expect("named additions compose");
    assert_eq!(
        composed.scheme_id_for_kind(PersistentNameSubjectKind::Entity, KindId(1)),
        Some("worth.compose.entity.v1")
    );
    assert!(composed.has_kind_name_schemes());
    let snapshot = composed.authority_snapshot();
    assert_eq!(snapshot.persistent_name_schemes.len(), 1);
    assert_eq!(snapshot.kind_name_schemes.len(), 1);
}

#[test]
fn schemaless_extension_leaves_no_naming_state() {
    let composed = RelationalSchemaRegistry::new()
        .extend(test_schema_registry())
        .expect("schemaless additions compose");
    assert!(composed.persistent_name_schemes().is_empty());
    assert!(!composed.has_kind_name_schemes());
    let snapshot = composed.authority_snapshot();
    assert!(snapshot.persistent_name_schemes.is_empty());
    assert!(snapshot.kind_name_schemes.is_empty());
}

#[test]
fn extension_never_replaces_an_installed_scheme_or_binding() {
    let composed = RelationalSchemaRegistry::new()
        .extend(named_additions())
        .unwrap();
    let kind_name = (*composed.owner_entity_kinds())[&KindId(1)]
        .kind_name
        .clone();

    let redefined = RelationalSchemaRegistry::new()
        .install_persistent_name_schemes(entity_scheme(&kind_name, ["other"]), BTreeMap::new())
        .unwrap();
    assert!(
        composed.clone().extend(redefined).is_err(),
        "a scheme identifier cannot change meaning by composition"
    );

    let identical = RelationalSchemaRegistry::new()
        .install_persistent_name_schemes(entity_scheme(&kind_name, ["authored"]), BTreeMap::new())
        .unwrap();
    let recomposed = composed
        .clone()
        .extend(identical)
        .expect("an identical scheme composes idempotently");
    assert_eq!(
        recomposed.authority_snapshot(),
        composed.authority_snapshot()
    );

    let rebound = composed.clone().extend_persistent_name_schemes(
        InstalledPersistentNameSchemeRegistry::new(),
        BTreeMap::from([(
            (PersistentNameSubjectKind::Entity, KindId(1)),
            "worth.compose.entity.v1".to_owned(),
        )]),
    );
    assert!(rebound.is_err(), "a bound kind cannot be bound again");
}
