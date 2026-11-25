//! Declaration validation and canonical identity tests for whole-result live
//! query subscriptions (`live_whole_result`).
use super::*;
use crate::application_schema::{
    ApplicationComponentSetRef, ApplicationEntityRef, U64ApplicationValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use crate::{worth_query_application_query, worth_query_structured_value_binding};

struct Schema;
struct Node;
struct Row;
struct Slot;
struct Set;

worth_query_portable_type!(Row => "worth.tests.live_whole.row.v1");
worth_query_portable_type!(Slot => "worth.tests.live_whole.slot.v1");
worth_query_structured_value_binding!(Params for () { identity: "worth.rust.unit" });
worth_query_structured_value_binding!(Rows for Row { identity: "worth.tests.live_whole.row.v1" });
worth_query_application_query!(
    Query for Schema,
    identity "worth.tests.live_whole.query.v1",
    parameters Params,
    result Rows,
    scope Node => "Node",
    name "live_whole_test"
);

fn builder(
    lanes: ApplicationQueryLaneEligibility,
) -> ApplicationQueryDefinitionBuilder<Schema, Query, (), Row, Node> {
    let node = ApplicationEntityRef::<Schema, Node>::from_schema_identifier("Node");
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(node)
        .scope(node)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Node, Row, Rows>::new(node).build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(lanes)
        .public()
}

fn live_cause() -> ApplicationQueryLiveCauseContract {
    ApplicationQueryLiveCauseContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryLiveCauseParts {
            binding_type: WorthQueryPortableTypeIdentity::declared(
                "worth.tests.live_whole.cause.v1",
            ),
            effect: "NodeChanged".to_owned(),
            payload_type: WorthQueryPortableTypeIdentity::declared("worth.rust.unit"),
            scope_slot_type: WorthQueryPortableTypeIdentity::declared(
                "worth.tests.live_whole.scope_slot.v1",
            ),
            scope_entity: "Node".to_owned(),
            scope_aspect: "NodeFacts".to_owned(),
            scope_field: "Key".to_owned(),
            scope_value_type: WorthQueryPortableTypeIdentity::declared("worth.rust.u64"),
            target_slot_type: WorthQueryPortableTypeIdentity::declared(
                "worth.tests.live_whole.target_slot.v1",
            ),
            target_entity: "Node".to_owned(),
            target_aspect: "NodeFacts".to_owned(),
            target_field: "Key".to_owned(),
            target_value_type: WorthQueryPortableTypeIdentity::declared("worth.rust.u64"),
            target_mode: ApplicationQueryLiveTargetMode::Collection,
        },
    )
}

#[test]
fn live_whole_result_validates_with_live_lane_and_changes_canonical_identity() {
    fn assert_auto_traits<
        T: Copy + Eq + Ord + Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
    >() {
    }
    assert_auto_traits::<ApplicationQueryLiveInvalidationMode>();
    let live = ApplicationQueryLaneEligibility::one_shot().with_live();
    assert_eq!(
        builder(live).build().unwrap_err(),
        ApplicationQueryDefinitionDenial::LiveLaneWithoutCauseContract
    );
    let one_shot = builder(ApplicationQueryLaneEligibility::one_shot())
        .build()
        .expect("one-shot query validates");
    let whole = builder(live)
        .live_whole_result()
        .build()
        .expect("whole-result live query validates");

    assert!(!one_shot.is_live_whole_result());
    assert_eq!(one_shot.live_invalidation_mode(), None);
    assert!(whole.is_live_whole_result());
    assert_eq!(
        whole.live_invalidation_mode(),
        Some(ApplicationQueryLiveInvalidationMode::WholeResult)
    );
    // Changing only `live_whole_result` alters the canonical basis identity.
    let mut parts_without_whole = whole.clone().into_erased().into_parts();
    parts_without_whole.live_whole_result = false;
    let erased_without_whole =
        ErasedApplicationQueryDefinition::from_untrusted_parts(parts_without_whole);
    assert_ne!(
        erased_without_whole.canonical_basis(),
        whole.into_erased().canonical_basis()
    );
}

#[test]
fn live_whole_result_rejects_one_shot_only_lane() {
    let err = builder(ApplicationQueryLaneEligibility::one_shot())
        .live_whole_result()
        .build()
        .unwrap_err();
    assert_eq!(
        err,
        ApplicationQueryDefinitionDenial::LiveCauseContractWithoutLane
    );
}

#[test]
fn live_whole_result_rejects_incompatible_shapes_on_fresh_validation() {
    let base = builder(ApplicationQueryLaneEligibility::one_shot().with_live())
        .live_whole_result()
        .build()
        .unwrap()
        .into_erased()
        .into_parts();

    // Combining with cause-routed live contract is refused.
    let mut with_cause = base.clone();
    with_cause.live_cause = Some(live_cause());
    assert_eq!(
        validate_portable_application_query_freshly(&with_cause),
        Err(ApplicationQueryDefinitionDenial::LiveWholeResultUnsupportedShape)
    );

    // Combining with continuation pagination is refused.
    let mut with_continuation = base.clone();
    with_continuation.continuation =
        Some(ApplicationQueryContinuationTarget::from_untrusted_parts(
            WorthQueryPortableApplicationQueryContinuationParts {
                query_type: base.query_type.clone(),
                slot_type: WorthQueryPortableTypeIdentity::declared(
                    "worth.tests.live_whole.cursor.v1",
                ),
                relation: "Next".to_owned(),
                parent_entity: "Node".to_owned(),
                child_entity: "Node".to_owned(),
                direction: ApplicationQueryResultTraversalDirection::Forward,
            },
        ));
    assert!(validate_portable_application_query_freshly(&with_continuation).is_err());

    // Combining with orbit is refused.
    let mut with_orbit = base.clone();
    with_orbit.orbit = Some(ApplicationQueryOrbitMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryOrbitParts {
            query: base.query_type.clone(),
            slot: WorthQueryPortableTypeIdentity::declared("worth.tests.live_whole.slot.v1"),
            relation: "Next".to_owned(),
            from: "Node".to_owned(),
            to: "Node".to_owned(),
            output_name: "orbit".to_owned(),
        },
    ));
    assert_eq!(
        validate_portable_application_query_freshly(&with_orbit),
        Err(ApplicationQueryDefinitionDenial::InvalidOrbitShape)
    );

    // Combining with component selection is refused.
    let mut with_component = base.clone();
    let set = ApplicationComponentSetRef::<Schema, Set, Node>::new(
        "Shells",
        ApplicationEntityRef::from_schema_identifier("Node"),
    );
    let comp =
        ApplicationQueryComponentRef::<Schema, Query, Slot, Set, Node>::census("shells", set);
    with_component.component_selection = Some(comp.meaning().clone());
    with_component.disclosure = ApplicationQueryDisclosureContract::public().components_by(
        comp,
        crate::application_schema::ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap(),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    assert_eq!(
        validate_portable_application_query_freshly(&with_component),
        Err(ApplicationQueryDefinitionDenial::InvalidComponentShape)
    );

    // Combining with managed index is refused.
    let mut with_index = base;
    with_index.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "Prefix".to_owned(),
            probe_parameter: "prefix".to_owned(),
        },
    ));
    assert_eq!(
        validate_portable_application_query_freshly(&with_index),
        Err(ApplicationQueryDefinitionDenial::InvalidManagedIndexShape)
    );
}
