//! What the Parent reads beside its Source outranks every scope exemption:
//! its fields and scope key on any entity, its kinds' lifecycle, and a new
//! scope answering to the probed scope's key.
use super::*;
use worth_foundational::facade::{AspectValue, FieldKey, InternedString, StructAspectValue};

/// A forward stored Source rooted only at the scope: other scopes root
/// their own Sources.
fn root_only() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let registry = event_registry();
    let (_, template) = managed_cache_schema();
    let mut source = template.owner_field_0().path.parts().clone();
    source.identity.name = "rooted-source".into();
    source.to = ROW.into();
    source.nodes = vec![Node::Stored {
        relation: INTRODUCES,
        from: SCOPE,
        to: ROW,
    }];
    let identity = source.identity.clone();
    let registry = registry.register_derived_relations(vec![source]).unwrap();
    let program = registry
        .bind_managed_index_source(
            &registry.derived_relation(&identity).unwrap(),
            vec![],
            vec![],
        )
        .unwrap();
    (registry, program)
}

fn key() -> (KindId, AspectKey, FieldKey) {
    (
        SCOPE,
        AspectKey::new("facts").unwrap(),
        FieldKey::new("key".to_string()).unwrap(),
    )
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}

/// An update whose aspect evidence keeps the old and new struct values.
fn updated(
    entity: EntityId,
    key_before: &str,
    key_after: &str,
    note_after: &str,
) -> CanonicalRecordAspectDelta {
    let (_, aspect, field) = key();
    let note = FieldKey::new("note".to_string()).unwrap();
    let value = |key: &str, note_value: &str| {
        StructAspectValue::new([(field.clone(), text(key)), (note.clone(), text(note_value))])
            .unwrap()
    };
    let mut delta = lifecycle(entity, Change::Updated);
    delta.changed_aspects = vec![aspect.clone()];
    delta.evaluated_bindings =
        smallvec::smallvec![crate::authority::mutation::EvaluatedAspectBinding {
            aspect_key: aspect.clone(),
            contract: worth_foundational::aspects()
                .contract()
                .for_key(aspect.clone())
                .identified_by(worth_foundational::AspectIdentity(9))
                .at_revision(worth_foundational::AspectContractRevision(1))
                .scalar(worth_foundational::ScalarAspectType::String),
            binding: worth_foundational::facade::AspectBinding::EntityField {
                field: field.clone()
            },
            changed: true,
            aspect_shape: worth_foundational::AspectShape::Scalar(
                worth_foundational::ScalarAspectType::String
            ),
            evidence: Evidence::StructAspectValueTransition {
                locator: worth_foundational::facade::AspectValueLocator::whole_aspect(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        aspect,
                    ),
                ),
                old_present: true,
                new_present: true,
                old_value: Some(value(key_before, "")),
                new_value: Some(value(key_after, note_after)),
            },
            field_revision_changes: None,
        }];
    delta
}

#[test]
fn the_scope_key_is_judged_field_by_field_on_every_scope() {
    let (registry, program) = root_only();
    let (probed, other) = (entity(1), entity(2));
    let effect = effect(Vec::new());
    let footprint = ManagedIndexParentReadFootprint {
        fields: vec![key()],
        scope_key: Some(key()),
        ..Default::default()
    };
    let relevance = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &unread);
    // A sibling field in the key's aspect stays aligned; the key does not,
    // on another scope as on the probed one.
    assert!(!relevance.delta(&updated(other, "other", "other", "note")));
    assert!(relevance.delta(&updated(other, "other", "peer", "")));
    assert!(relevance.delta(&updated(probed, "peer", "renamed", "")));
    let none = ManagedIndexParentReadFootprint::default();
    let source_only = SourceRelevance::of(&registry, &program, probed, &none, &effect, &unread);
    assert!(!source_only.delta(&updated(other, "other", "peer", "")));
}

#[test]
fn a_parent_read_scope_kind_keeps_every_lifecycle_change() {
    let (registry, program) = root_only();
    let (probed, other) = (entity(1), entity(2));
    let effect = effect(Vec::new());
    let none = ManagedIndexParentReadFootprint::default();
    let source_only = SourceRelevance::of(&registry, &program, probed, &none, &effect, &unread);
    assert!(!source_only.delta(&lifecycle(other, Change::Deleted)));
    let read = ManagedIndexParentReadFootprint {
        entities: vec![SCOPE],
        ..Default::default()
    };
    let parent = SourceRelevance::of(&registry, &program, probed, &read, &effect, &unread);
    for change in [
        Change::Created,
        Change::Deleted,
        Change::MaterializationSuspended,
    ] {
        assert!(parent.delta(&lifecycle(other, change)), "{change:?}");
    }
}

#[test]
fn a_new_scope_answering_to_the_probed_key_is_relevant() {
    let (registry, program) = root_only();
    let (probed, colliding, distinct, unreadable) = (entity(1), entity(2), entity(3), entity(4));
    let effect = effect(Vec::new());
    let footprint = ManagedIndexParentReadFootprint {
        scope_key: Some(key()),
        ..Default::default()
    };
    let keys = |entity: EntityId| match entity.local_slot_value() {
        1 | 2 => Some(text("peer")),
        3 => Some(text("sibling")),
        _ => None,
    };
    let relevance = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &keys);
    assert!(relevance.delta(&lifecycle(colliding, Change::Created)));
    assert!(relevance.delta(&lifecycle(unreadable, Change::Created)));
    assert!(!relevance.delta(&lifecycle(distinct, Change::Created)));
    assert!(!relevance.delta(&lifecycle(distinct, Change::Deleted)));
}
