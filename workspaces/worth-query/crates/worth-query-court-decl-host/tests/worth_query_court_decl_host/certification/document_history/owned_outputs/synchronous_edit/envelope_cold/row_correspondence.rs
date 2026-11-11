//! Match every original Native row by domain name or its single fixture metadata role.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::{
    identity::EntityId, publication::PublishedAuthoritativeRecordPatch,
};
type Published = BTreeMap<String, (RecordRef, RecordStructuralChange)>;
type Metadata = BTreeMap<String, (Option<String>, AspectValue)>;
pub(super) fn observe(
    patches: &[PublishedAuthoritativeRecordPatch],
    rows: &BTreeMap<String, (EntityId, u64)>,
    envelope_key: &[u8; 32],
    expected_records: usize,
) -> (Published, Metadata) {
    let envelope_key = envelope_key
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut metadata = Metadata::new();
    let mut entity_roles = BTreeMap::new();
    for patch in patches {
        let RecordRef::Entity(id) = patch.target else {
            continue;
        };
        if entity_roles.contains_key(&id) {
            continue;
        }
        let role = match rows.iter().find(|(_, (entity, _))| *entity == id) {
            Some((name, _)) => format!("entity:{name}"),
            None => metadata_role(patch, &mut metadata, &envelope_key),
        };
        assert!(
            entity_roles.insert(id, role).is_none(),
            "each published entity has one semantic name"
        );
    }
    let mut published = Published::new();
    for patch in patches {
        let role = match patch.target {
            RecordRef::Entity(id) => entity_roles[&id].clone(),
            RecordRef::Relation(_) => {
                let birth = patch
                    .relation_birth
                    .expect("this envelope creates its named edges");
                format!(
                    "relation:{:?}:{}->{}",
                    birth.kind,
                    entity_roles
                        .get(&birth.source)
                        .or_else(|| rows
                            .iter()
                            .find(|(_, (id, _))| *id == birth.source)
                            .map(|(name, _)| name))
                        .expect("named source"),
                    entity_roles
                        .get(&birth.target)
                        .or_else(|| rows
                            .iter()
                            .find(|(_, (id, _))| *id == birth.target)
                            .map(|(name, _)| name))
                        .expect("named target")
                )
            }
        };
        match published.entry(role) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((patch.target.clone(), patch.structural_change));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                assert_eq!(
                    entry.get().0,
                    patch.target,
                    "distinct rows cannot share a semantic role"
                );
                assert!(
                    matches!(
                        patch.structural_change,
                        RecordStructuralChange::Created | RecordStructuralChange::Updated
                    ),
                    "this envelope retires no row"
                );
                if patch.structural_change == RecordStructuralChange::Created {
                    entry.get_mut().1 = RecordStructuralChange::Created;
                }
            }
        }
    }
    assert_eq!(
        published
            .values()
            .filter(|(_, change)| *change == RecordStructuralChange::Created)
            .count(),
        published.len() - 1
    );
    // The receipt counts native patch fragments. Several fragments may name
    // one metadata row; neither a fragment nor a physical row may disappear.
    assert_eq!(
        patches.len(),
        expected_records,
        "Native's original changed-record count covers every patch fragment"
    );
    assert_eq!(
        published.len(),
        patches
            .iter()
            .map(|patch| &patch.target)
            .collect::<BTreeSet<_>>()
            .len(),
        "every distinct original record has one mapped role"
    );
    (published, metadata)
}
fn metadata_role(
    patch: &PublishedAuthoritativeRecordPatch,
    metadata: &mut Metadata,
    envelope_key: &str,
) -> String {
    if let Some(value) = patch
        .authoritative_patch
        .struct_set_for(&AspectKey::new("idempotency").unwrap())
    {
        let text = |field| match value
            .get(&FieldKey::new(field).unwrap())
            .expect("original receipt field")
        {
            AspectValue::String(InternedString::Raw(text)) => text.clone(),
            other => panic!("canonical receipt field {field}: {other:?}"),
        };
        let envelope = text("key") == envelope_key;
        let role = if envelope {
            "idempotency:envelope"
        } else {
            "idempotency:first-producer"
        }
        .to_owned();
        // The sole producer's input identity includes per-run candidate IDs.
        // Keep its exact original intent in the printed patches. Its semantic
        // meaning is checked by a fresh real route demand with zero execution;
        // never strip or rewrite opaque identity slots to force text equality.
        assert!(metadata
            .insert(
                role.clone(),
                (
                    envelope.then(|| text("intent")),
                    value
                        .get(&FieldKey::new("emitted-effect-count").unwrap())
                        .unwrap()
                        .clone()
                )
            )
            .is_none());
        return role;
    }
    // This fixture produces exactly one route with one member and one edge.
    // Private receipt/witness payloads have per-run IDs and remain Native's
    // grammar. Reopen and public demand exercise their real owner readmission.
    let aspects = patch
        .authoritative_changed_aspect_keys()
        .map(|key| key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(aspects.len(), 1, "unmatched metadata: {patch:?}");
    assert!(
        [
            "worth-current-publication-catalog",
            "worth-current-publication",
            "worth-current-publication-edge",
            "worth-current-output-position"
        ]
        .contains(&aspects[0]),
        "unmatched original entity: {patch:?}"
    );
    format!("native:{}", aspects[0])
}

// Native issues one Create lineage for each newly created entity here. Match
// those recorded lineages in their actual creation order, with exact bijection
// and no extra events, while record IDs are matched separately by semantic role.
pub(super) fn lineages(
    warm: &[worth_relational::facade::lineage::LineageEventRecord],
    cold: &[worth_relational::facade::lineage::LineageEventRecord],
    created_entities: usize,
) {
    use worth_relational::facade::lineage::LineageEventKind;
    assert_eq!(warm.len(), created_entities);
    assert_eq!(cold.len(), created_entities);
    let mut ids = BTreeMap::new();
    for (a, b) in warm.iter().zip(cold) {
        for event in [a, b] {
            assert_eq!(event.kind(), LineageEventKind::Create);
            assert!(event.sources().is_empty());
            assert_eq!(event.targets().len(), 1);
        }
        assert_eq!(a.branch_id(), b.branch_id());
        assert!(ids.insert(a.targets()[0], b.targets()[0]).is_none());
    }
    assert_eq!(
        ids.values().collect::<BTreeSet<_>>().len(),
        created_entities
    );
}

/// Every original published record has exactly one counterpart by role.
pub(super) fn records(warm: &Published, cold: &Published) {
    assert_eq!(
        warm.keys().collect::<Vec<_>>(),
        cold.keys().collect::<Vec<_>>()
    );
    let bijection = warm
        .iter()
        .map(|(name, (id, change))| {
            let (other, other_change) = &cold[name];
            assert_eq!(change, other_change);
            (id.clone(), other.clone())
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(bijection.len(), warm.len());
    assert_eq!(
        bijection.values().collect::<BTreeSet<_>>().len(),
        cold.len(),
        "exact bijection covers every published row without unmatched or extra rows"
    );
}

pub(super) fn named_values(
    warm: &BTreeMap<String, (EntityId, u64)>,
    cold: &BTreeMap<String, (EntityId, u64)>,
) {
    assert_eq!(
        warm.iter()
            .map(|(name, (_, value))| (name, value))
            .collect::<Vec<_>>(),
        cold.iter()
            .map(|(name, (_, value))| (name, value))
            .collect::<Vec<_>>()
    );
}
