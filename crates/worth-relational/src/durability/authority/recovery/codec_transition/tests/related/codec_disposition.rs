//! Same-route birth after an actual encoded governed retirement; claims cannot substitute.
use super::{
    super::fixture::{basis, capture},
    fixture::*,
};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, RecordRef, RevalidateEntityIntent, WorkerIntentBatch,
};

#[derive(Clone, Copy)]
enum ClaimDefect {
    SiblingKind,
    RevalidationWrite,
}

#[test]
fn cold_retirement_requires_every_sibling_group_before_route_eligibility() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed(1);
    let world = world_with_installation(execution, runtime, receipt.clone());
    let (owner, publisher) = receipt.issue_application_publication_owner();
    let sibling = super::super::fixture::create(
        execution,
        &world.runtime,
        3,
        "independent sibling",
        "nondefault sibling output",
        Some((
            &publisher,
            owner.admit_required_output("native.sibling.producer", [23; 32]),
        )),
    );
    let (mut target, mut recovered, _) = recover(execution, &world);
    let candidate = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        crate::mvcc::RelationalOperationControl::uninterrupted(),
        Some(
            WorkerIntentBatch::new("judge protected source without rewriting it").push(
                MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
                    entity_id: world.authored,
                })),
            ),
        ),
    )
    .unwrap();
    let (_, recovered) = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap()
        .into_parts();
    recovered.admit_basis(basis(&target)).unwrap();
    let saved = capture(&target);
    let decoded =
        crate::durability::log::native_file_codec::decode_checkpoint(execution, saved.bytes())
            .unwrap()
            .checkpoint;
    let mut hidden_write = decoded
        .envelopes
        .iter()
        .flat_map(|entry| &entry.patch.authoritative_record_patches)
        .find(|patch| patch.target == RecordRef::Entity(world.authored))
        .unwrap()
        .clone();
    hidden_write.structural_change =
        crate::publication::patch::data::RecordStructuralChange::Updated;
    for defect in [
        None,
        Some(ClaimDefect::SiblingKind),
        Some(ClaimDefect::RevalidationWrite),
    ] {
        let mut changed = decoded.clone();
        let entry = changed
            .envelopes
            .iter_mut()
            .find(|entry| !entry.required_output_codec_retirements.is_empty())
            .unwrap();
        assert_eq!(entry.required_output_codec_retirements.len(), 2);
        if matches!(defect, Some(ClaimDefect::SiblingKind)) {
            let sibling = entry
                .envelope_mut_for_test()
                .required_output_codec_retirements
                .iter_mut()
                .find(|event| event.meaning.family == "native.closed.producer")
                .unwrap();
            // A valid A cannot authorize B's deletions from B's header alone.
            std::sync::Arc::make_mut(&mut std::sync::Arc::make_mut(sibling).kinds).pop();
        }
        if matches!(defect, Some(ClaimDefect::RevalidationWrite)) {
            // An actual read demand cannot authorize an equal-payload patch.
            entry
                .envelope_mut_for_test()
                .patch
                .authoritative_record_patches
                .push(hidden_write.clone());
            let required =
                crate::symbols::data::required_commit_symbols(&entry.merged_plan, &entry.patch);
            // Keep the hostile claim structurally current: its actual source
            // values retain their original native symbol meanings.
            entry.envelope_mut_for_test().symbol_texts.entries = required
                .iter()
                .map(|symbol| {
                    decoded
                        .symbol_table
                        .entries
                        .iter()
                        .find(|(id, _)| id == symbol)
                        .unwrap()
                        .clone()
                })
                .collect();
        }
        assert!(entry.codec_retirement_headers_valid());
        let (bytes, _) = crate::durability::log::native_file_codec::encode_checkpoint(
            &changed,
            execution.allocation_policy(),
        )
        .unwrap();
        let len = bytes.len();
        let checkpoint =
            crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes_region(
                bytes,
                0..len,
            )
            .unwrap();
        let (mut cold, receipt) = installed(2);
        cold.durability_recovery()
            .restore_native_checkpoint(execution, &checkpoint)
            .unwrap();
        let receipt = cold
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let before = capture(&cold);
        let eligibility = cold.readmit_required_output_route_absence(
            execution,
            &installed,
            &basis(&cold),
            &crate::mvcc::native_meaning_fixture("native.sibling.producer"),
        );
        if defect.is_some() {
            assert!(
                eligibility.is_err(),
                "a malformed sibling or hidden write must deny the valid route"
            );
            assert_eq!(capture(&cold), before);
        } else {
            eligibility.unwrap();
            assert!(!basis(&cold)
                .inner
                .root
                .partition_state(sibling.partition_id)
                .unwrap()
                .entity_arena
                .get(&sibling)
                .unwrap()
                .is_live());
        }
    }
}

pub(super) fn cold_retired_route(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &RelationalRuntime,
) {
    let saved = capture(target);
    let (mut cold, receipt) = installed(2);
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (outputs, relations) = produce(execution, &cold, &receipt, 2);
    super::super::fixture::build_index(execution, &cold);
    assert_order(&cold, &outputs, &relations);
    assert_values(
        &cold,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    let selected = basis(&cold);
    let commit = cold
        .history
        .canonical_envelope(selected.inner.root.commit_id().unwrap())
        .unwrap();
    assert!(matches!(
        commit.required_output_origin.as_ref().unwrap().publication,
        crate::history::data::CanonicalOutputPublication::BirthAfterCodecRetirement { .. }
    ));

    let decoded =
        crate::durability::log::native_file_codec::decode_checkpoint(execution, saved.bytes())
            .unwrap()
            .checkpoint;
    let foreign = decoded
        .envelopes
        .iter()
        .find(|entry| {
            entry.required_output_origin.is_none()
                && entry.required_output_codec_retirements.is_empty()
        })
        .unwrap()
        .commit
        .commit_id;
    for incomplete in [false, true] {
        let mut changed = decoded.clone();
        let entry = changed
            .envelopes
            .iter_mut()
            .find(|entry| !entry.required_output_codec_retirements.is_empty())
            .unwrap();
        let event = std::sync::Arc::make_mut(
            &mut entry
                .envelope_mut_for_test()
                .required_output_codec_retirements[0],
        );
        if incomplete {
            std::sync::Arc::make_mut(&mut event.kinds).pop(); // Actual protected program kind was last.
        } else {
            event.publication = foreign;
        }
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
        let (mut rejected, receipt) = installed(2);
        rejected
            .durability_recovery()
            .restore_native_checkpoint(execution, &untrusted)
            .unwrap();
        let receipt = rejected
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let before = capture(&rejected);
        let denied = rejected
            .readmit_required_output_route_absence(
                execution,
                &installed,
                &basis(&rejected),
                &crate::mvcc::native_meaning_fixture("native.closed.producer"),
            )
            .unwrap_err();
        let expected = if incomplete {
            "actual protected program contract"
        } else {
            "no installed producer publication"
        };
        assert!(format!("{denied:?}").contains(expected), "{denied:?}");
        assert_eq!(capture(&rejected), before);
    }
}
