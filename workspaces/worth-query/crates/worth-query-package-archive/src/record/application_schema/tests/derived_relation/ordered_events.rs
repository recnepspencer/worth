//! Typed event syntax survives canonical transport and fresh schema validation.
use super::*;
use crate::facade::WorthQueryPackageArchiveDenialKind as ArchiveKind;
use worth_query_declaration::facade::application_schema::*;
struct Events;
struct Event;
struct Row;
impl ApplicationEntityMarkerIdentity<Events> for Event {
    const IDENTIFIER: &'static str = "Event";
}
impl ApplicationEntityMarkerIdentity<Events> for Row {
    const IDENTIFIER: &'static str = "Row";
}

struct Previous;
struct Introduces;
struct Retires;
struct Live;
impl ApplicationSchema for Events {
    const OWNER: &'static str = "archive.events";
    const NAME: &'static str = "Events";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, Denial> {
        let event = ApplicationEntityRef::<Self, Event>::from_schema_identifier("Event");
        let row = ApplicationEntityRef::<Self, Row>::from_schema_identifier("Row");
        let (previous, introduces, retires) = leaves();
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(event)
            .entity(row)
            .relation(previous, event, event)
            .relation(introduces, event, row)
            .relation(retires, event, row)
            .derived_relation(reference(), recipe(), Laws::unbounded())
            .build()
    }
}
fn reference() -> ApplicationDerivedRelationRef<Events, Live, Event, Row> {
    ApplicationDerivedRelationRef::from_schema_identifiers("Live", "Event", "Row")
}
fn leaves() -> (
    ApplicationRelationRef<Events, Previous, Event, Event>,
    ApplicationRelationRef<Events, Introduces, Event, Row>,
    ApplicationRelationRef<Events, Retires, Event, Row>,
) {
    let integrity = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    (
        ApplicationRelationRef::from_schema_identifiers("Previous", "Event", "Event", integrity),
        ApplicationRelationRef::from_schema_identifiers("Introduces", "Event", "Row", integrity),
        ApplicationRelationRef::from_schema_identifiers("Retires", "Event", "Row", integrity),
    )
}
fn recipe() -> ApplicationDerivedRelation<Events, Event, Row> {
    let (previous, introduces, retires) = leaves();
    ApplicationDerivedRelation::ordered_events(previous, introduces, retires)
}
fn member() -> Member {
    Events::declaration()
        .unwrap()
        .erased()
        .members()
        .iter()
        .find(|m| matches!(m, Member::DerivedRelation { .. }))
        .unwrap()
        .clone()
}
fn definition(member: &Member) -> &Definition {
    let Member::DerivedRelation { definition } = member else {
        unreachable!()
    };
    definition
}
fn record(members: Vec<Member>) -> Schema {
    Schema::from_untrusted_parts(Parts {
        owner: Events::OWNER.into(),
        name: Events::NAME.into(),
        major: 1,
        minor: 0,
        members,
        contributions: vec![],
    })
}
#[test]
fn typed_ordered_events_have_fresh_versioned_transport_and_complete_physical_closure() {
    let original = member();
    let bytes = encode_member(&original);
    assert_eq!(&bytes[..4], &[0, 47, 0, 2]);
    assert_eq!(decode_member(&bytes), original);
    let schema = Events::declaration().unwrap();
    let projected = Schema::project(schema.erased());
    let restored = projected
        .members()
        .iter()
        .map(|m| decode_member(&encode_member(m)))
        .collect();
    let admitted = readmit(record(restored)).unwrap();
    assert_eq!(admitted.identity(), readmit(projected).unwrap().identity());
    let closure = application_derived_relation_source_closure(admitted.members(), "Live").unwrap();
    assert!(closure.has_extended_expression);
    assert!(closure.projected_domains.is_empty());
    let names = closure
        .physical
        .iter()
        .map(|source| source.relation.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["Introduces", "Previous", "Retires"]);
    let mut swapped = definition(&original).parts().clone();
    swapped.expression.nodes[3] = Node::OrderedEvents {
        predecessor: 0,
        introduced: 2,
        retired: 1,
    };
    let mut members = admitted.members().to_vec();
    *members
        .iter_mut()
        .find(|m| matches!(m, Member::DerivedRelation { .. }))
        .unwrap() = Member::DerivedRelation {
        definition: Definition::from_untrusted_parts(swapped),
    };
    assert_ne!(
        readmit(record(members)).unwrap().identity(),
        admitted.identity()
    );
}
#[test]
fn typed_event_composition_offsets_all_roles_and_retains_native_order() {
    let (previous, _, _) = leaves();
    let expression = ApplicationDerivedRelation::compose(previous, recipe()).into_expression();
    assert!(matches!(
        expression.nodes[4],
        Node::OrderedEvents {
            predecessor: 1,
            introduced: 2,
            retired: 3
        }
    ));
    assert!(matches!(
        expression.nodes[5],
        Node::Compose { left: 0, right: 4 }
    ));
    assert_eq!(
        expression.validate().unwrap(),
        (
            ApplicationGraphMemberSelector::Entity("Event"),
            ApplicationGraphMemberSelector::Entity("Row")
        )
    );
    let reversed = recipe().reversed().into_expression();
    assert_eq!(
        reversed.validate().unwrap(),
        (
            ApplicationGraphMemberSelector::Entity("Row"),
            ApplicationGraphMemberSelector::Entity("Event")
        )
    );
}
#[test]
fn event_transport_rejects_stale_unknown_and_noncanonical_versions() {
    let mut bytes = encode_member(&member());
    bytes[2..4].copy_from_slice(&1u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        ArchiveKind::UnsupportedRecordVariant
    );
    bytes[2..4].copy_from_slice(&3u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        ArchiveKind::UnsupportedRecordVariant
    );
    let old_extended = derived(
        "Both",
        vec![
            source(SourceKind::Stored, "Edge"),
            source(SourceKind::Stored, "Edge"),
            Node::Union { left: 0, right: 1 },
        ],
    );
    let mut bytes = encode_member(&old_extended);
    assert_eq!(&bytes[..4], &[0, 47, 0, 1]);
    let mut expected = vec![0, 47, 0, 1];
    let text = |bytes: &mut Vec<u8>, value: &str| {
        bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    };
    text(&mut expected, "Both");
    for _ in 0..2 {
        expected.extend_from_slice(&1u16.to_be_bytes());
        text(&mut expected, "E");
    }
    expected.extend_from_slice(&3u32.to_be_bytes());
    for _ in 0..2 {
        expected.extend_from_slice(&1u16.to_be_bytes());
        expected.extend_from_slice(&1u16.to_be_bytes());
        text(&mut expected, "Edge");
        for _ in 0..2 {
            expected.extend_from_slice(&1u16.to_be_bytes());
            text(&mut expected, "E");
        }
    }
    expected.extend_from_slice(&6u16.to_be_bytes());
    expected.extend_from_slice(&0u64.to_be_bytes());
    expected.extend_from_slice(&1u64.to_be_bytes());
    for word in [3u16, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0] {
        expected.extend_from_slice(&word.to_be_bytes());
    }
    assert_eq!(bytes, expected, "literal pre-event tag47 version1 bytes");
    bytes[2..4].copy_from_slice(&2u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        ArchiveKind::InvalidRecordShape
    );
}
#[test]
fn untrusted_event_roles_reject_nonliteral_projected_bad_indices_and_missing_sources() {
    let original = member();
    for mode in 0..6 {
        let mut parts = definition(&original).parts().clone();
        let expected = match mode {
            0 => {
                parts.expression.nodes[3] = Node::OrderedEvents {
                    predecessor: 3,
                    introduced: 1,
                    retired: 2,
                };
                Cause::InvalidNodeReference
            }
            1 => {
                parts.expression.nodes[0] = Node::Reverse { input: 0 };
                Cause::InvalidNodeReference
            }
            2 => {
                let Node::Source(source) = &mut parts.expression.nodes[0] else {
                    unreachable!()
                };
                source.kind = SourceKind::Derived;
                Cause::InvalidEventOperand
            }
            3 => {
                let Node::Source(source) = &mut parts.expression.nodes[2] else {
                    unreachable!()
                };
                source.to = "Event".into();
                Cause::DiscontinuousEndpoints
            }
            4 => {
                let Node::Source(source) = &mut parts.expression.nodes[2] else {
                    unreachable!()
                };
                source.to = ApplicationGraphMemberMeaning::Projected("Row".into());
                Cause::MissingSource
            }
            _ => {
                parts.expression.nodes.insert(3, Node::Reverse { input: 0 });
                parts.expression.nodes[4] = Node::OrderedEvents {
                    predecessor: 3,
                    introduced: 1,
                    retired: 2,
                };
                Cause::InvalidEventOperand
            }
        };
        let invalid = Definition::from_untrusted_parts(parts);
        assert_eq!(invalid.validate(), Err(expected), "hostile mode{mode}");
        let mut output = crate::binary_output::BinaryOutput::with_capacity(0);
        assert_eq!(
            super::super::super::member::write(
                &mut output,
                &Member::DerivedRelation {
                    definition: invalid
                }
            )
            .unwrap_err()
            .kind(),
            ArchiveKind::InvalidRecordShape
        );
        assert!(output.into_bytes().is_empty());
    }
    let mut members = Events::declaration().unwrap().erased().members().to_vec();
    members.retain(|m| !matches!(m,Member::Relation{relation,..} if relation=="Retires"));
    assert!(matches!(
        readmit(record(members)),
        Err(Denial::InvalidApplicationDerivedRelation {
            cause: Cause::MissingSource,
            ..
        })
    ));
}
