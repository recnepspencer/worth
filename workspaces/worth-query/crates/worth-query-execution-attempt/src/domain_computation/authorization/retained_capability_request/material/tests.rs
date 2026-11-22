//! Representation-only tests: these requests carry no issued grant or Source.
use super::super::context::RetainedContext;
use super::*;
use worth_query_declaration::facade::application_capability::WorthQueryPortableApplicationCapabilityRelationBindingParts as RelationParts;
use worth_relational::facade::identity::PartitionId;
fn entity(slot: u64, generation: u32) -> EntityId {
    EntityId::new(PartitionId::main(), slot, generation)
}
fn key(parts: [&str; 5]) -> WorthQueryCapabilityContextKey {
    serde_json::from_value(serde_json::json!({ "context":parts[0], "context_type":parts[1], "slot":parts[2], "slot_type":parts[3], "entity":parts[4] })).unwrap()
}
fn relation(parts: [&str; 3]) -> ApplicationCapabilityRelationBinding {
    ApplicationCapabilityRelationBinding::from_untrusted_parts(RelationParts {
        relation: parts[0].into(),
        from: parts[1].into(),
        to: parts[2].into(),
    })
}
fn request() -> WorthQueryRetainedCapabilityRequest {
    WorthQueryRetainedCapabilityRequest {
        recorded_actor: None,
        capability_identity: [7; 32],
        principal: entity(1, 1),
        resource: entity(2, 1),
        data: Arc::new(RetainedCapabilityRequestData {
            _codec_funding: None,
            resource_entity: Arc::from("resource"),
            elevation: Some(entity(3, 1)),
            action: AspectValue::String("a".into()),
            purpose: AspectValue::String("bc".into()),
            related_relation: Some(relation(["related", "resource", "subject"])),
            related: Some(entity(4, 1)),
            field: Some(AspectValue::UInt64(5)),
            magnitude: Some(AspectValue::UInt64(6)),
            cardinality: 7,
            context_name: Arc::from("context"),
            context_type: Arc::from("type"),
            context: RetainedContext::Native(BTreeMap::from([(
                key(["context", "type", "peer", "entity-type", "subject"]),
                entity(8, 1),
            )])),
        }),
    }
}
fn digest(request: &WorthQueryRetainedCapabilityRequest) -> [u8; 32] {
    descriptor(request, &mut |_| Ok::<_, ()>(()), &|| ()).unwrap()
}
#[test]
fn request_descriptor_binds_every_use_dimension_and_full_generations() {
    let original = request();
    let expected = digest(&original);
    assert_eq!(expected, digest(&request()));
    for axis in 0..22 {
        let mut changed = request();
        match axis {
            0 => changed.capability_identity = [8; 32],
            1 => changed.principal = entity(1, 2),
            2 => changed.resource = entity(2, 2),
            axis => {
                let data = Arc::get_mut(&mut changed.data).unwrap();
                match axis {
                    3 => data.resource_entity = Arc::from("different"),
                    4 => data.elevation = Some(entity(3, 2)),
                    5 => data.action = AspectValue::String("different".into()),
                    6 => data.purpose = AspectValue::String("different".into()),
                    7..=9 => {
                        let mut parts = ["related", "resource", "subject"];
                        parts[axis - 7] = "different";
                        data.related_relation = Some(relation(parts));
                    }
                    10 => data.related = Some(entity(4, 2)),
                    11 => data.field = None,
                    12 => data.magnitude = Some(AspectValue::String("6".into())),
                    13 => data.cardinality = 8,
                    14 => data.context_name = Arc::from("different"),
                    15 => data.context_type = Arc::from("different"),
                    16..=20 => {
                        let mut parts = ["context", "type", "peer", "entity-type", "subject"];
                        parts[axis - 16] = "different";
                        data.context =
                            RetainedContext::Native(BTreeMap::from([(key(parts), entity(8, 1))]));
                    }
                    21 => {
                        data.context = RetainedContext::Native(BTreeMap::from([(
                            key(["context", "type", "peer", "entity-type", "subject"]),
                            entity(8, 2),
                        )]))
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert!(!original.matches_use(&changed), "axis {axis}");
        assert_ne!(expected, digest(&changed), "axis {axis}");
    }
}
#[test]
fn descriptor_frames_values_and_normalizes_only_context_backing() {
    let original = request();
    let mut changed = request();
    let data = Arc::get_mut(&mut changed.data).unwrap();
    data.action = AspectValue::String("ab".into());
    data.purpose = AspectValue::String("c".into());
    assert_ne!(digest(&original), digest(&changed));
    let mut decoded = request();
    let data = Arc::get_mut(&mut decoded.data).unwrap();
    data.context = RetainedContext::decoded(vec![(
        key(["context", "type", "peer", "entity-type", "subject"]),
        entity(8, 1),
    )])
    .unwrap();
    assert!(original.matches_use(&decoded));
    assert_eq!(digest(&original), digest(&decoded));
}
#[test]
fn descriptor_keeps_owned_first_stop_and_final_poll_without_text_buffer() {
    #[derive(Debug, PartialEq, Eq)]
    struct Stop(String);
    let mut calls = 0;
    let stopped = descriptor(
        &request(),
        &mut |_| {
            calls += 1;
            Err(Stop("first".into()))
        },
        &|| Stop("overflow".into()),
    );
    assert_eq!(stopped, Err(Stop("first".into())));
    assert_eq!(calls, 1);
    let stopped = descriptor(
        &request(),
        &mut |work| {
            if work == 0 {
                Err(Stop("final".into()))
            } else {
                Ok(())
            }
        },
        &|| Stop("overflow".into()),
    );
    assert_eq!(stopped, Err(Stop("final".into())));
}
