//! Local catalog selection cost; publication is exercised by installed journeys.

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::installed_layout;

#[test]
fn unrelated_aspects_preserve_selected_inventory() {
    let mut layout = installed_layout();
    let selected = layout
        .native_output_aspects("Account")
        .cloned()
        .collect::<Vec<_>>();
    assert!(!selected.is_empty());
    let kind = layout.entity_kind("Account");
    let contracts = selected
        .iter()
        .map(|aspect| {
            (
                aspect.clone(),
                layout.aspect_contract("Account", aspect).unwrap().clone(),
            )
        })
        .collect::<Vec<_>>();
    let original_count: usize = layout.aspect_contracts.values().map(BTreeMap::len).sum();
    let original_kind_count = layout.entity_kinds.len();
    let unrelated = layout.aspect_contracts.get_mut("Activity").unwrap();
    let contract = unrelated.values().next().unwrap().clone();
    // This expands only a local collection, not the installed schema or a World.
    for index in 0..4096 {
        assert!(unrelated
            .insert(
                AspectKey::new(format!("unrelated-{index:04}")).unwrap(),
                contract.clone(),
            )
            .is_none());
    }
    assert_eq!(
        layout
            .aspect_contracts
            .values()
            .map(BTreeMap::len)
            .sum::<usize>(),
        original_count + 4096
    );
    assert_eq!(layout.entity_kinds.len(), original_kind_count);
    assert_eq!(layout.entity_kind("Account"), kind);
    assert_eq!(
        layout
            .native_output_aspects("Account")
            .cloned()
            .collect::<Vec<_>>(),
        selected
    );
    for (aspect, contract) in contracts {
        assert_eq!(layout.aspect_contract("Account", &aspect), Some(&contract));
    }
    assert_eq!(layout.native_output_aspects("AbsentEntity").count(), 0);
    assert!(layout.entity_kind("AbsentEntity").is_none());
}
