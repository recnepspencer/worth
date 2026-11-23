//! Genuine recorded headers with one closed native corruption per court.
use super::*;

#[test]
fn restoration_refuses_a_revived_anchor() {
    check(
        true,
        "invalid recorded authored restoration anchor",
        Corruption::RevivedAnchor,
    );
}

#[test]
fn restoration_refuses_an_empty_publication_roster() {
    check(
        true,
        "invalid recorded authored restoration anchor",
        Corruption::EmptyPublicationRoster,
    );
}

#[test]
fn restoration_refuses_a_missing_anchor_mapping() {
    check(
        true,
        "invalid recorded authored restoration anchor",
        Corruption::MissingAnchorMapping,
    );
}

#[test]
fn restoration_refuses_an_anchor_without_a_create_patch() {
    check(
        true,
        "invalid recorded authored restoration anchor",
        Corruption::AnchorWithoutCreatePatch,
    );
}

#[test]
fn restoration_refuses_repeated_co_anchors() {
    check(
        true,
        "restoration repeats or omits a co-anchor",
        Corruption::RepeatedCoAnchors,
    );
}

#[test]
fn restoration_refuses_a_missing_co_anchor_mapping() {
    check(
        true,
        "restoration repeats or omits a co-anchor",
        Corruption::MissingCoAnchorMapping,
    );
}

#[test]
fn restoration_refuses_duplicate_entity_generations() {
    check(
        true,
        "recorded restoration repeats or revives a native generation",
        Corruption::DuplicateEntityGenerations,
    );
}

#[test]
fn restoration_refuses_revived_entity_generations() {
    check(
        true,
        "recorded restoration repeats or revives a native generation",
        Corruption::RevivedEntityGenerations,
    );
}

#[test]
fn restoration_refuses_repeated_relation_generations() {
    check(
        true,
        "restoration repeats or invents a relation allocation",
        Corruption::RepeatedRelationGenerations,
    );
}

#[test]
fn restoration_refuses_a_relation_without_a_create_patch() {
    check(
        true,
        "restoration repeats or invents a relation allocation",
        Corruption::RelationWithoutCreatePatch,
    );
}

#[test]
fn restoration_refuses_incomplete_origin_membership() {
    check(
        true,
        "copied or incomplete native output publication membership",
        Corruption::IncompleteOriginMembership,
    );
}

#[test]
fn restoration_refuses_an_unowned_publication() {
    check(
        true,
        "restoration omits exact recorded members",
        Corruption::UnownedPublication,
    );
}

#[test]
fn restoration_refuses_a_missing_member_mapping() {
    check(
        true,
        "restoration omits exact recorded members",
        Corruption::MissingMemberMapping,
    );
}

#[test]
fn restoration_refuses_a_missing_relation_inverse() {
    check(
        true,
        "restoration relation has no recorded inverse",
        Corruption::MissingRelationInverse,
    );
}

#[test]
fn restoration_refuses_a_repeated_route() {
    check(
        true,
        "restoration repeats an original route or member",
        Corruption::RepeatedRoute,
    );
}

#[test]
fn restoration_refuses_a_repeated_output_member() {
    check(
        true,
        "restoration repeats an original route or member",
        Corruption::RepeatedOutputMember,
    );
}

#[test]
fn restoration_refuses_a_foreign_source() {
    check(
        true,
        "restored publication has a foreign source",
        Corruption::ForeignSource,
    );
}

#[test]
fn restoration_refuses_a_cyclic_source() {
    check(
        true,
        "restored source ownership is cyclic",
        Corruption::CyclicSource,
    );
}

#[test]
fn restoration_refuses_an_unowned_relation() {
    check(
        true,
        "restoration adds an unowned relation",
        Corruption::UnownedRelation,
    );
}

#[test]
fn restoration_refuses_an_unowned_entity() {
    check(
        true,
        "restoration adds an unowned entity",
        Corruption::UnownedEntity,
    );
}
