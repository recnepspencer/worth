//! Setup resolves actual compiled ownership, with typed absence and meaning refusals.
use super::*;
use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind as QueryKind;
type Schema = model::ParentSchema<0>;
type Index = queries::ModelIndex<Schema>;

struct MissingIndex;
impl ApplicationManagedIndexBinding<Schema> for MissingIndex {
    type SourceBinding = queries::SourceBinding<Schema>;
    type ProbeBinding = ProbeBinding;
    type Entity = model::Principal<Schema>;
    const NAME: &'static str = "MissingPrefix";
    const PROVIDER: &'static str = Index::PROVIDER;
    const PROVIDER_VERSION: u32 = Index::PROVIDER_VERSION;
}

#[test]
fn typed_lookup_classifies_absence_drift_and_actual_catalog_ownership() {
    let schema = bind::<0>().unwrap();
    let selected = schema.installed_managed_index::<Index>().unwrap();
    let reissued = schema.installed_managed_index::<Index>().unwrap();
    assert!(selected.same_installation(&reissued));
    assert!(selected.is_installed_in(&schema));
    assert!(selected.admits_parent(
        &schema
            .installed_query_binding::<queries::ParentBinding<Schema>>()
            .unwrap(),
    ));

    let foreign_schema = bind::<0>().unwrap();
    let foreign = foreign_schema.installed_managed_index::<Index>().unwrap();
    assert_eq!(selected.definition(), foreign.definition());
    assert!(!selected.same_installation(&foreign));
    assert!(!selected.is_installed_in(&foreign_schema));
    assert!(!selected.admits_parent(
        &foreign_schema
            .installed_query_binding::<queries::ParentBinding<Schema>>()
            .unwrap(),
    ));

    let Err(missing) = schema.installed_managed_index::<MissingIndex>() else {
        panic!("a genuinely absent index must refuse");
    };
    assert_eq!(missing.kind(), QueryKind::InvalidManagedIndexDefinition);
    assert_eq!(missing.subject(), MissingIndex::NAME);

    let Err(changed) = schema.installed_managed_index::<queries::OtherIndex<Schema>>() else {
        panic!("equal names cannot substitute an incompatible typed probe recipe");
    };
    assert_eq!(changed.kind(), QueryKind::ManagedIndexBindingMeaningChanged);
    assert_eq!(changed.subject(), Index::NAME);
}
