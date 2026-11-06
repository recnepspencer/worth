use super::{BTreeSet, InstalledProgramTarget};

#[test]
fn optional_aspect_removal_keeps_its_entity_and_aspect_in_declared_and_installed_views() {
    let (declared, installed) = worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::aspect_removal::declared_and_installed_targets();
    let aspect = worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::aspect_removal::optional_aspect_name();
    assert_eq!(
        declared,
        BTreeSet::from([format!("remove-aspect:Subject/{aspect}")])
    );
    assert_eq!(
        installed,
        BTreeSet::from([InstalledProgramTarget::RemoveAspect {
            entity: "Subject".to_owned(),
            aspect: aspect.to_owned(),
        }])
    );
}
