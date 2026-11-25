//! Forged relation rosters are refused through ordinary checkpoint recovery.
use super::{
    super::super::checkpoint_wire_fixture::{edit_original, field, replace_field},
    super::{
        fixture::{basis, capture},
        publication_fixture, serial_request,
    },
    fixture::*,
    incoming_reference::{installed_references, relation_payload},
    origin::{exact_origin, RelationOrigin},
};
use worth_relational::facade::{
    history::CommitId, identity::KindId, mvcc::RequiredOutputRolePosture,
};
mod incidental_birth;

#[test]
fn cold_boundary_rosters_refuse_missing_owned_edge_and_foreign_sibling_claim() {
    refuse_missing_and_foreign();
    incidental_birth::refuse_late_claim();
}

fn edit_relations(bytes: &[u8], edit: impl FnOnce(&mut Vec<RelationOrigin>)) -> Vec<u8> {
    let origin = field(bytes, "required_output_origin");
    let mut relations = rmp_serde::from_slice(field(origin, "relations")).unwrap();
    edit(&mut relations);
    let amended = replace_field(
        origin,
        "relations",
        &rmp_serde::to_vec_named(&relations).unwrap(),
    );
    replace_field(bytes, "required_output_origin", &amended)
}

fn refuse_missing_and_foreign() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_references(1);
    let (world, owned) = world_with_boundary(execution, runtime, receipt.clone());
    let (boundary, publisher) = owned.unwrap();
    let publication = publisher.custody;
    let (_, _, sibling_edge, sibling) = produce_with_boundary(
        execution,
        &world.runtime,
        &receipt,
        1,
        world.outputs[1],
        "native.sibling.producer",
    );
    let foreign = exact_origin(execution, &world.runtime, &sibling.custody)
        .relations
        .iter()
        .find(|entry| entry.id == sibling_edge)
        .unwrap()
        .clone();
    let saved = capture(execution, &world.runtime);
    let commit = CommitId(publication.authority().publication_coordinate().0);
    for missing in [true, false] {
        let untrusted = edit_original(&saved, commit, |bytes| {
            edit_relations(bytes, |relations| {
                if missing {
                    relations.retain(|entry| entry.id != boundary);
                } else {
                    relations.push(foreign.clone());
                    relations.sort_unstable_by_key(|entry| entry.id);
                }
            })
        });
        let (mut cold, receipt) = installed_references(1);
        cold.durability_recovery()
            .restore_native_checkpoint(execution, &untrusted, worth_testkit_relational::cold_recovery::members_source(&untrusted))
            .unwrap();
        let receipt = cold
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let selected = basis(&cold);
        let before = capture(execution, &cold);
        let denied = cold
            .readmit_required_output_publication(
                execution,
                &installed,
                &selected,
                "native.closed.producer",
                &[19; 32],
                &publication_fixture::meaning("native.closed.producer"),
                &publication_fixture::roles(
                    &world.outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Create,
                ),
            )
            .unwrap_err();
        let expected = if missing {
            "missing from its birth roster"
        } else {
            "relation birth differs"
        };
        assert!(
            format!("{denied:?}").contains(expected),
            "missing={missing}: {denied:?}"
        );
        assert!(cold
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&selected)
            .is_ok());
        assert_eq!(capture(execution, &cold), before);
        assert_eq!(
            relation_payload(execution, &cold, boundary),
            relation_payload(execution, &world.runtime, boundary)
        );
    }
}
