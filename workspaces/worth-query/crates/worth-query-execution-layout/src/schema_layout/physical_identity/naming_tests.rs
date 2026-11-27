//! Original readmission compares installed persistent-name meaning too: a
//! recorded installation whose schemes or kind bindings differ refuses.
use std::collections::BTreeMap;

use super::value_layout::{lower, schema};
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, PersistentNameSubjectKind,
};

#[test]
fn original_readmission_refuses_differing_persistent_name_schemes() {
    let original = lower(&schema(false, false, false), None);
    assert!(original
        .roles
        .require_original(&original.registry, &original.registry)
        .is_ok());
    let zulu = original.roles.entities["Zulu"];
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                "worth.tests.zulu.v1",
                PersistentNameSubjectKind::Entity,
                "Zulu",
                ["authored"],
                false,
                false,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    let named = original
        .registry
        .clone()
        .install_persistent_name_schemes(
            schemes,
            BTreeMap::from([(
                (PersistentNameSubjectKind::Entity, zulu),
                "worth.tests.zulu.v1".to_owned(),
            )]),
        )
        .unwrap();
    assert!(
        original
            .roles
            .require_original(&named, &original.registry)
            .is_err(),
        "a declaration adding a scheme cannot readmit an unschemed original"
    );
    assert!(
        original
            .roles
            .require_original(&original.registry, &named)
            .is_err(),
        "a declaration dropping a scheme cannot readmit a schemed original"
    );
}
