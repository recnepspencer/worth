//! Contract revisions follow portable meaning rather than declaration traversal or Rust placement.
use std::marker::PhantomData;
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_schema::{fragment::*, *};
mod records;
mod schema;
use records::*;
use schema::*;

macro_rules! lower {
    ($set:ty, $members:ty) => {
        ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Root::<$set, $members>(
            PhantomData,
        ))
        .unwrap()
        .build()
        .unwrap()
    };
}
macro_rules! rows {
    ($set:ty, $row:ty) => { lower!($set, Members<$set, $row>) };
}
type Declaration = ApplicationSchemaDeclaration<Schema>;
#[derive(Debug, Eq, PartialEq)]
struct Revisions {
    owned: AspectContractRevision,
    extension: AspectContractRevision,
    payload: AspectContractRevision,
    extension_payload: AspectContractRevision,
    case: AspectContractRevision,
}
fn entity_revision(schema: &Declaration, key: &str) -> AspectContractRevision {
    schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Aspect {
                aspect, revision, ..
            } if aspect == key => Some(*revision),
            _ => None,
        })
        .expect("the declared aspect was lowered")
}
fn payload_revision(schema: &Declaration, relation: &str) -> AspectContractRevision {
    schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::RelationAspect { definition }
                if definition.parts().relation == relation =>
            {
                Some(definition.parts().revision)
            }
            _ => None,
        })
        .expect("the declared payload was lowered")
}
fn keys() -> [&'static str; 3] {
    let owner: Owner<Standard, Model<Row>> = Owner(PhantomData);
    let extension: Extras<Standard, Model<Row>, Row> = Extensions(PhantomData);
    let item: Model<Row> = Item(PhantomData);
    let owned = aspect(owner, item, Facts::<Row>(PhantomData)).name();
    let item: Model<Row> = Item(PhantomData);
    let extended = aspect(extension, item, ExtraFacts::<Row>(PhantomData)).name();
    let item: Model<Row> = Item(PhantomData);
    let owner: Owner<Standard, Model<Row>> = Owner(PhantomData);
    let case = aspect(owner, item, Case::<Row>(PhantomData)).name();
    [owned, extended, case]
}
fn revisions(schema: &Declaration) -> Revisions {
    let [owned, extension, case] = keys();
    Revisions {
        owned: entity_revision(schema, owned),
        extension: entity_revision(schema, extension),
        payload: payload_revision(schema, "owned_link"),
        extension_payload: payload_revision(schema, "extended_link"),
        case: entity_revision(schema, case),
    }
}
fn assert_all_move(base: &Declaration, changed: &Declaration) {
    let base = revisions(base);
    let changed = revisions(changed);
    assert_ne!(base.owned, changed.owned, "owned");
    assert_ne!(base.extension, changed.extension, "extension");
    assert_ne!(base.payload, changed.payload, "owned relation payload");
    assert_ne!(
        base.extension_payload, changed.extension_payload,
        "extension relation payload"
    );
    assert_ne!(base.case, changed.case, "one-of case");
}
fn entity_fields(schema: &Declaration, key: &str) -> Vec<ApplicationSchemaMember> {
    schema
        .erased()
        .members()
        .iter()
        .filter(|member| {
            matches!(member,
        ApplicationSchemaMember::Field { aspect, .. } if aspect == key)
        })
        .cloned()
        .collect()
}
fn payload_fields(schema: &Declaration, relation: &str) -> Vec<ApplicationRelationFieldDefinition> {
    schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::RelationAspect { definition }
                if definition.parts().relation == relation =>
            {
                Some(definition.parts().fields.clone())
            }
            _ => None,
        })
        .unwrap()
}
fn assert_only_group_moves(base: &Declaration, changed: &Declaration) {
    assert_eq!(
        entity_fields(base, keys()[2]),
        entity_fields(changed, keys()[2]),
        "the case's own fields must stay identical"
    );
    let mut expected = revisions(base);
    let actual = revisions(changed);
    assert_ne!(expected.case, actual.case);
    expected.case = actual.case;
    assert_eq!(expected, actual, "unrelated contracts must stay identical");
}
#[test]
fn lowering_the_same_declaration_twice_keeps_every_aspect_revision() {
    assert_eq!(
        revisions(&Schema::declaration().unwrap()),
        revisions(&Schema::declaration().unwrap())
    );
}
#[test]
fn field_declaration_order_keeps_owned_extension_payload_and_case_revisions() {
    assert_eq!(
        revisions(&rows!(Standard, Row)),
        revisions(&rows!(Standard, ReorderedRow))
    );
}
#[test]
fn fragment_composition_order_keeps_owned_extension_payload_and_case_revisions() {
    let reordered = lower!(
        Standard,
        (
            Extras<Standard, Model<Row>, Row>,
            Owner<Standard, Model<Row>>
        )
    );
    assert_eq!(
        revisions(&Schema::declaration().unwrap()),
        revisions(&reordered)
    );
}
#[test]
fn renaming_a_field_moves_owned_extension_payload_and_case_revisions() {
    assert_all_move(&rows!(Standard, Row), &rows!(Standard, RenamedFieldRow));
}
#[test]
fn changing_only_a_scalar_codec_identity_moves_every_aspect_revision() {
    assert_eq!(
        U64ApplicationValueBinding::SCALAR_FAMILY,
        AlternateU64::SCALAR_FAMILY
    );
    assert_ne!(U64ApplicationValueBinding::IDENTITY, AlternateU64::IDENTITY);
    assert_all_move(&rows!(Standard, Row), &rows!(Alternate, Row));
}
#[test]
fn making_a_field_optional_or_required_moves_every_aspect_revision() {
    let required = rows!(Standard, Row);
    let optional = rows!(Standard, OptionalFieldRow);
    assert_all_move(&required, &optional);
}
#[test]
fn aspect_optionality_moves_owned_and_both_relation_payload_revisions() {
    type E =
        Item<(Option<Facts<Row>>, Exclusive<Choice<Row, (Case<Row>,)>>), (Option<Payload<Row>>,)>;
    let changed = lower!(
        Standard,
        (
            Owner<Standard, E>,
            Extensions<Standard, E, ExtraFacts<Row>, (Option<Payload<Row>>,)>
        )
    );
    let original = rows!(Standard, Row);
    assert_eq!(
        entity_fields(&original, keys()[0]),
        entity_fields(&changed, keys()[0]),
        "only aspect absence changed"
    );
    for relation in ["owned_link", "extended_link"] {
        assert_eq!(
            payload_fields(&original, relation),
            payload_fields(&changed, relation),
            "only payload absence changed"
        );
    }
    let base = revisions(&original);
    let changed = revisions(&changed);
    assert_ne!(base.owned, changed.owned);
    assert_ne!(base.payload, changed.payload);
    assert_ne!(base.extension_payload, changed.extension_payload);
    assert_eq!(
        base.extension, changed.extension,
        "extensions remain optional by construction"
    );
    assert_eq!(base.case, changed.case);
}
#[test]
fn changing_the_one_of_roster_moves_the_unchanged_case_revision() {
    type E = Item<(Facts<Row>, Exclusive<Choice<Row, (Case<Row>, OtherCase)>>), (Payload<Row>,)>;
    let changed = lower!(Standard, (Owner<Standard, E>, Extras<Standard, E, Row>));
    assert_only_group_moves(&rows!(Standard, Row), &changed);
}
#[test]
fn changing_one_of_optionality_moves_the_unchanged_case_revision() {
    type E = Item<(Facts<Row>, Option<Exclusive<Choice<Row, (Case<Row>,)>>>), (Payload<Row>,)>;
    let changed = lower!(Standard, (Owner<Standard, E>, Extras<Standard, E, Row>));
    assert_only_group_moves(&rows!(Standard, Row), &changed);
}
#[test]
fn one_of_case_order_keeps_the_same_case_revision() {
    type Forward =
        Item<(Facts<Row>, Exclusive<Choice<Row, (Case<Row>, OtherCase)>>), (Payload<Row>,)>;
    type Reversed =
        Item<(Facts<Row>, Exclusive<Choice<Row, (OtherCase, Case<Row>)>>), (Payload<Row>,)>;
    let forward = lower!(
        Standard,
        (Owner<Standard, Forward>, Extras<Standard, Forward, Row>)
    );
    let reversed = lower!(
        Standard,
        (Owner<Standard, Reversed>, Extras<Standard, Reversed, Row>)
    );
    assert_eq!(revisions(&forward), revisions(&reversed));
}
#[test]
fn renamed_rust_aspect_markers_with_the_same_names_keep_every_revision() {
    type E = Item<
        (
            RenamedFacts<Row>,
            Exclusive<Choice<Row, (RenamedCase<Row>,)>>,
        ),
        (RenamedPayload<Row>,),
    >;
    let renamed = lower!(
        Standard,
        (
            Owner<Standard, E>,
            Extensions<Standard, E, RenamedExtra<Row>, (RenamedPayload<Row>,)>
        )
    );
    assert_eq!(revisions(&rows!(Standard, Row)), revisions(&renamed));
}
#[test]
fn an_unrelated_sibling_aspect_keeps_every_existing_revision() {
    type E = Item<(Facts<Row>, Exclusive<Choice<Row, (Case<Row>,)>>, Unrelated), (Payload<Row>,)>;
    let changed = lower!(Standard, (Owner<Standard, E>, Extras<Standard, E, Row>));
    assert_eq!(revisions(&rows!(Standard, Row)), revisions(&changed));
}
