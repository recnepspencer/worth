//! Hostile checkpoint claims cannot create relation birth ownership.
use super::*;
pub(super) fn refuse_missing_and_foreign() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = super::super::incoming_reference::installed_references(1);
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
    let foreign = sibling
        .custody
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .find(|entry| entry.id == sibling_edge)
        .unwrap()
        .clone();
    let saved = capture(&world.runtime);
    let decoded =
        crate::durability::log::native_file_codec::decode_checkpoint(execution, saved.bytes())
            .unwrap()
            .checkpoint;
    for missing in [true, false] {
        let mut changed = decoded.clone();
        edit_publication(
            &mut changed,
            publication.envelope.commit.commit_id,
            |envelope| {
                // Hostile claims are changed only inside an actual native checkpoint input.
                let mut origin: crate::history::data::CanonicalRequiredOutputOrigin =
                    rmp_serde::from_slice(
                        &rmp_serde::to_vec(
                            envelope.required_output_origin.as_ref().unwrap().as_ref(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
                if missing {
                    origin.relations.retain(|entry| entry.id != boundary);
                } else {
                    origin.relations.push(foreign.clone());
                    origin.relations.sort_unstable_by_key(|entry| entry.id);
                }
                envelope.required_output_origin = Some(std::sync::Arc::new(origin));
            },
        );
        let (bytes, _) = crate::durability::log::native_file_codec::encode_checkpoint(
            &changed,
            execution.allocation_policy(),
        )
        .unwrap();
        let len = bytes.len();
        let untrusted =
            crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes_region(
                bytes,
                0..len,
            )
            .unwrap();
        let (mut cold, receipt) = super::super::incoming_reference::installed_references(1);
        cold.durability_recovery()
            .restore_native_checkpoint(execution, &untrusted)
            .unwrap();
        let receipt = cold
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let selected = basis(&cold);
        let before = capture(&cold);
        let denied = cold
            .readmit_required_output_publication(
                execution,
                &installed,
                &selected,
                "native.closed.producer",
                &[19; 32],
                &crate::mvcc::native_meaning_fixture("native.closed.producer"),
                &crate::mvcc::native_roles_fixture(
                    &world
                        .outputs
                        .map(|id| (id, crate::identity::data::KindId(3))),
                    crate::mvcc::RequiredOutputRolePosture::Create,
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
        assert!(selected.is_current());
        assert_eq!(capture(&cold), before);
        assert_eq!(
            relation_payload(&cold, boundary),
            relation_payload(&world.runtime, boundary)
        );
    }
}
