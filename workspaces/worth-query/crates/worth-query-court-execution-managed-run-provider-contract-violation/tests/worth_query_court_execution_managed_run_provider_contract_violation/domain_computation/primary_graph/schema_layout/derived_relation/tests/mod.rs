//! Actual installed Query meaning binds exact Native roles and physical audience.
use worth_query_declaration::facade::application_schema::ApplicationGraphMemberMeaning;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::installed_layout::installed_schema_layout;
use worth_query_declaration::facade::application_schema::*;
use worth_relational::facade::schema::DerivedRelationNode as Native;
pub struct Events;
pub struct Event;
pub struct Row;

pub struct Previous;
pub struct Introduces;
pub struct Retires;
pub struct Live;
pub struct PreviousLive;
impl ApplicationSchema for Events {
    const OWNER: &'static str = "query.events.binding";
    const NAME: &'static str = "Events";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
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
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(EventRecords)?
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
    let event = layout.entity_kind_for_test("Event").unwrap();
    let row = layout.entity_kind_for_test("Row").unwrap();
    let previous = (*layout.relation("Previous").unwrap().kind_for_test());
    let introduces = (*layout.relation("Introduces").unwrap().kind_for_test());
    let retires = (*layout.relation("Retires").unwrap().kind_for_test());
    let live = layout.derived_relation("Live").unwrap();
    assert_eq!(
        (*live.handle_for_test()).parts().nodes,
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
        (*live.from_for_test()),
        ApplicationGraphMemberMeaning::Entity("Event".into())
    );
    assert_eq!(
        (*live.to_for_test()),
        ApplicationGraphMemberMeaning::Entity("Row".into())
    );
    let expected = std::collections::BTreeSet::from([
        (previous, event, event),
        (introduces, event, row),
        (retires, event, row),
    ]);
    assert_eq!(
        (*live.handle_for_test())
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
    assert!((*live.carriers_for_test()).is_empty());
    assert_eq!(
        registry
            .derived_relation(&(*live.handle_for_test()).parts().identity)
            .unwrap(),
        (*live.handle_for_test()),
        "lowering retains the actual registry issuer, not a rebuilt equal recipe"
    );
    let composed = layout.derived_relation("PreviousLive").unwrap();
    assert!(matches!(
        (*composed.handle_for_test()).parts().nodes[4],
        Native::OrderedEvents {
            predecessor: 1,
            introduced: 2,
            retired: 3
        }
    ));
    assert!(matches!(
        (*composed.handle_for_test()).parts().nodes[5],
        Native::Compose { left: 0, right: 4 }
    ));
}

use worth_query_declaration::facade::application_schema::fragment;
struct EventRecords;
impl fragment::Fragment for EventRecords {
    const NAME: &'static str = "event_records";
    type Bindings = fragment::NoBindings;
    type Members = (Event, Row);
}
impl fragment::Entity for Event {
    const NAME: &'static str = "Event";
    type Aspects = ();
    type Relations = (Previous, Introduces, Retires);
}
impl fragment::Entity for Row {
    const NAME: &'static str = "Row";
    type Aspects = ();
    type Relations = ();
}
impl fragment::Relation for Previous {
    const NAME: &'static str = "Previous";
    type To = Event;
}
impl fragment::Relation for Introduces {
    const NAME: &'static str = "Introduces";
    type To = Row;
}
impl fragment::Relation for Retires {
    const NAME: &'static str = "Retires";
    type To = Row;
}
