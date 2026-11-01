use super::{BTreeSet, InstalledProgramTarget};

#[test]
fn optional_aspect_removal_keeps_its_entity_and_aspect_in_declared_and_installed_views() {
    let (declared, installed) = worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::aspect_removal::declared_and_installed_targets();
    assert_eq!(
        declared,
        BTreeSet::from(["remove-aspect:Subject/OptionalFacts".to_owned()])
    );
    assert_eq!(
        installed,
        BTreeSet::from([InstalledProgramTarget::RemoveAspect {
            entity: "Subject".to_owned(),
            aspect: "OptionalFacts".to_owned(),
        }])
    );
}
