//! Frozen aggregate slot0/1 and the new descriptive component slot2.
use super::*;
use worth_query_declaration::{
    facade::{application_query::*, application_schema::*},
    *,
};
struct Schema;
struct Node;
struct Row;
struct Slot;
struct Set;
worth_query_portable_type!(Row=>"worth.tests.archive.component-row.v1");
worth_query_portable_type!(Slot=>"worth.tests.archive.component-slot.v1");
worth_query_structured_value_binding!(ParametersBinding for () {identity:"worth.rust.unit"});
worth_query_structured_value_binding!(RowsBinding for Row {identity:"worth.tests.archive.component-row.v1"});
worth_query_application_query!(Query for Schema,identity "worth.tests.archive.component-query.v1",parameters ParametersBinding,result RowsBinding,scope Node=>"Node",name "components");
fn parts() -> WorthQueryPortableApplicationQueryParts {
    let node = ApplicationEntityRef::<Schema, Node>::from_schema_identifier("Node");
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(node)
        .scope(node)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Node, Row, RowsBinding>::new(node)
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
        .into_erased()
        .into_parts()
}
fn component(
    mode: ApplicationQueryComponentSelection,
) -> ApplicationQueryComponentRef<Schema, Query, Slot, Set, Node> {
    let set = ApplicationComponentSetRef::new(
        "Shells",
        ApplicationEntityRef::from_schema_identifier("Node"),
    );
    match mode {
        ApplicationQueryComponentSelection::OfScopeMember => {
            ApplicationQueryComponentRef::of_scope_member("shells", set)
        }
        ApplicationQueryComponentSelection::Census => {
            ApplicationQueryComponentRef::census("shells", set)
        }
    }
}
fn orbit() -> ApplicationQueryOrbitMeaning {
    ApplicationQueryOrbitMeaning::from_untrusted_parts(WorthQueryPortableApplicationQueryOrbitParts{query:parts().query_type,slot:worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared("slot"),relation:"Next".into(),from:"Node".into(),to:"Node".into(),output_name:"orbit".into()})
}
#[test]
fn aggregate_slot_none_and_orbit_keep_exact_original_wire_bytes() {
    let mut original = parts();
    for has_orbit in [false, true] {
        original.orbit = has_orbit.then(orbit);
        let mut out = crate::binary_output::BinaryOutput::with_capacity(256);
        root_selection::choice::write(&mut out, &original).unwrap();
        let mut expected = crate::binary_output::BinaryOutput::with_capacity(256);
        expected.u16(u16::from(has_orbit));
        if has_orbit {
            for text in [
                "worth.tests.archive.component-query.v1",
                "slot",
                "Next",
                "Node",
                "Node",
                "orbit",
            ] {
                expected.text(text);
            }
        }
        assert_eq!(out.into_bytes(), expected.into_bytes());
    }
}
#[test]
fn component_query_and_whole_slot_disclosure_round_trip_for_both_modes() {
    for mode in [
        ApplicationQueryComponentSelection::OfScopeMember,
        ApplicationQueryComponentSelection::Census,
    ] {
        let mut source = parts();
        source.component_selection = Some(component(mode).meaning().clone());
        source.root_paths = vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: "Node".into(),
                terminal_entity: "Node".into(),
                steps: vec![],
                guards: vec![],
            },
        )];
        source.disclosure = ApplicationQueryDisclosureContract::public().components_by(
            component(mode),
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap(),
            ApplicationQueryInfluenceContract::permit_all(),
        );
        let query = ErasedApplicationQueryDefinition::from_untrusted_parts(source);
        let mut out = crate::binary_output::BinaryOutput::with_capacity(1024);
        write(&mut out, &query).unwrap();
        let bytes = out.into_bytes();
        let mut input = BinaryInput::new(&bytes);
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(decode(&mut input, &mut budget).unwrap(), query);
        assert!(input.is_finished());
    }
}
#[test]
fn ambiguous_aggregate_refuses_preflight_and_write_before_bytes() {
    let mut source = parts();
    source.orbit = Some(orbit());
    source.component_selection = Some(
        component(ApplicationQueryComponentSelection::Census)
            .meaning()
            .clone(),
    );
    let query = ErasedApplicationQueryDefinition::from_untrusted_parts(source);
    assert!(require_nesting_depth(&query, 32).is_err());
    let mut out = crate::binary_output::BinaryOutput::with_capacity(256);
    assert!(write(&mut out, &query).is_err());
    assert!(out.into_bytes().is_empty());
}

#[test]
fn committed_index_choice_v1_round_trips_and_keeps_legacy_component_bytes() {
    let mut source = parts();
    source.component_selection = Some(
        component(ApplicationQueryComponentSelection::Census)
            .meaning()
            .clone(),
    );
    let mut output = crate::binary_output::BinaryOutput::with_capacity(256);
    root_selection::choice::write(&mut output, &source).unwrap();
    let mut expected = crate::binary_output::BinaryOutput::with_capacity(256);
    expected.u16(2);
    for text in [
        "worth.tests.archive.component-query.v1",
        "worth.tests.archive.component-slot.v1",
        "Shells",
        "Node",
        "shells",
    ] {
        expected.text(text);
    }
    expected.u16(2);
    assert_eq!(output.into_bytes(), expected.into_bytes());
    source.component_selection = None;
    source.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "Prefix".into(),
            probe_parameter: "prefix".into(),
        },
    ));
    let mut output = crate::binary_output::BinaryOutput::with_capacity(256);
    root_selection::choice::write(&mut output, &source).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..4], &[0, 3, 0, 1]);
    let mut input = BinaryInput::new(&bytes);
    let (decoded_orbit, component, index) = root_selection::choice::decode(&mut input).unwrap();
    assert!(decoded_orbit.is_none() && component.is_none() && input.is_finished());
    assert_eq!(index, source.managed_index);
    let mut input = BinaryInput::new(&[0, 3, 0, 2]);
    assert_eq!(
        root_selection::choice::decode(&mut input)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    source.orbit = Some(orbit());
    let mut output = crate::binary_output::BinaryOutput::with_capacity(256);
    assert!(root_selection::choice::write(&mut output, &source).is_err());
    assert!(output.into_bytes().is_empty());
}
