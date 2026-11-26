//! Tree-based undo's companion sets (public #56, ruling (u)) leave the
//! canonical encoding of every selection that moves no reversal fact
//! unchanged, and an envelope written before them decodes as "no
//! companions", never as damage.
use super::*;
use crate::identity::data::PartitionId;

/// A descriptor of an empty main branch, as an observed basis serializes
/// it. The envelope only carries it; this test reads no branch.
const DESCRIPTOR: &str = r#"{"descriptor_version":2,"runtime_instance_id":1,"branch_id":"main","reference":{"branch_id":"relational/1/main","target":"Empty","generation":0},"truth_version":0,"root_identity":0,"schema_commitment":"d88918c5d7c1c31bc3df72d45eac0c683488dbff018bca1726e03b7004685707","visibility_commitment":"0000000000000000000000000000000000000000000000000000000000000000","posture":"Live"}"#;

fn selection(companion_retired: Vec<EntityId>, companion_selected: Vec<SelectedEntity>) -> CanonicalOwnedGraphSelection {
    CanonicalOwnedGraphSelection::from_observed(
        serde_json::from_str(DESCRIPTOR).unwrap(),
        EntityId::new(PartitionId::main(), 1, 1),
        vec![EntityId::new(PartitionId::main(), 2, 1)],
        Vec::new(),
        vec![SelectedEntity { id: EntityId::new(PartitionId::main(), 2, 1), kind: KindId(1) }],
        Vec::new(),
        Vec::new(),
        companion_retired,
        companion_selected,
    )
}

#[test]
fn a_selection_without_companions_keeps_its_encoding() {
    let encoded = serde_json::to_value(selection(Vec::new(), Vec::new())).unwrap();
    let fields = encoded.as_object().unwrap();
    assert!(!fields.contains_key("companion_retired"), "{encoded}");
    assert!(!fields.contains_key("companion_selected"), "{encoded}");
}

#[test]
fn an_envelope_written_before_companions_decodes_as_none() {
    let fact = EntityId::new(PartitionId::main(), 7, 3);
    let carried = selection(vec![fact], vec![SelectedEntity { id: EntityId::new(PartitionId::main(), 8, 2), kind: KindId(5) }]);
    let mut encoded = serde_json::to_value(&carried).unwrap();
    let decoded: CanonicalOwnedGraphSelection = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, carried);
    assert_eq!(decoded.companion_retired().collect::<Vec<_>>(), vec![fact]);
    let fields = encoded.as_object_mut().unwrap();
    fields.remove("companion_retired");
    fields.remove("companion_selected");
    let old: CanonicalOwnedGraphSelection = serde_json::from_value(encoded).unwrap();
    assert_eq!(old.companion_retired().count(), 0);
    assert_eq!(old.companion_selected().count(), 0);
    assert_eq!(old, selection(Vec::new(), Vec::new()));
    assert_eq!(old.targets().count(), 2, "an old selection targets only its owned records");
}
