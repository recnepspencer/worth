//! Actual installed Query meaning binds exact Native roles and physical audience.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::installed_schema_layout;
use worth_query_declaration::facade::application_schema::*;
use worth_relational::facade::schema::DerivedRelationNode as Native;
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
struct PreviousLive;
impl ApplicationSchema for Events {
    const OWNER: &'static str = "query.events.binding";
    const NAME: &'static str = "Events";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let event = ApplicationEntityRef::<Self, Event>::from_schema_identifier("Event");
        let row = ApplicationEntityRef::<Self, Row>::from_schema_identifier("Row");
        let integrity = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        let previous =
            ApplicationRelationRef::<Self, Previous, Event, Event>::from_schema_identifiers(
                "Previous", "Event", "Event", integrity,
            );
        let introduces =
            ApplicationRelationRef::<Self, Introduces, Event, Row>::from_schema_identifiers(
                "Introduces",
                "Event",
                "Row",
                integrity,
            );
        let retires = ApplicationRelationRef::<Self, Retires, Event, Row>::from_schema_identifiers(
            "Retires", "Event", "Row", integrity,
        );
        let live = ApplicationDerivedRelationRef::<Self, Live, Event, Row>::from_schema_identifiers(
            "Live", "Event", "Row",
        );
        let prior = ApplicationDerivedRelationRef::<Self,PreviousLive,Event,Row>::from_schema_identifiers("PreviousLive","Event","Row");
        let recipe = || ApplicationDerivedRelation::ordered_events(previous, introduces, retires);
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(event)
            .entity(row)
            .relation(previous, event, event)
            .relation(introduces, event, row)
            .relation(retires, event, row)
            .derived_relation(live, recipe(), ApplicationRelationGraphLaws::unbounded())
            .derived_relation(
                prior,
                ApplicationDerivedRelation::compose(previous, recipe()),
                ApplicationRelationGraphLaws::unbounded(),
            )
            .build()
    }
}
#[test]
fn typed_ordered_event_schema_installs_exact_native_roles_and_full_physical_roster() {
    let (layout, registry) = installed_schema_layout::<Events>();
    let event = layout.entity_kind("Event").unwrap();
    let row = layout.entity_kind("Row").unwrap();
    let previous = layout.relation("Previous").unwrap().kind;
    let introduces = layout.relation("Introduces").unwrap().kind;
    let retires = layout.relation("Retires").unwrap().kind;
    let live = layout.derived_relation("Live").unwrap();
    assert_eq!(
        live.handle.parts().nodes,
        vec![
            Native::Stored {
                relation: previous,
                from: event,
                to: event
            },
            Native::Stored {
                relation: introduces,
                from: event,
                to: row
            },
            Native::Stored {
                relation: retires,
                from: event,
                to: row
            },
            Native::OrderedEvents {
                predecessor: 0,
                introduced: 1,
                retired: 2
            },
        ]
    );
    assert_eq!(
        live.from,
        ApplicationGraphMemberMeaning::Entity("Event".into())
    );
    assert_eq!(live.to, ApplicationGraphMemberMeaning::Entity("Row".into()));
    let expected = std::collections::BTreeSet::from([
        (previous, event, event),
        (introduces, event, row),
        (retires, event, row),
    ]);
    assert_eq!(
        live.handle
            .physical_sources()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        live.sources
            .iter()
            .map(|(_, kind, from, to)| (*kind, *from, *to))
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    assert!(live.carriers.is_empty());
    assert_eq!(
        registry
            .derived_relation(&live.handle.parts().identity)
            .unwrap(),
        live.handle,
        "lowering retains the actual registry issuer, not a rebuilt equal recipe"
    );
    let composed = layout.derived_relation("PreviousLive").unwrap();
    assert!(matches!(
        composed.handle.parts().nodes[4],
        Native::OrderedEvents {
            predecessor: 1,
            introduced: 2,
            retired: 3
        }
    ));
    assert!(matches!(
        composed.handle.parts().nodes[5],
        Native::Compose { left: 0, right: 4 }
    ));
}
