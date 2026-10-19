//! An actual external-to-external birth cannot become output custody by later rewiring.
use super::*;
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::identity::data::{EntityId, KindId};
use crate::mvcc::{
    RelationalPublicationOutcome, RelationalTransactionIntent, RequiredOutputPublication,
    RequiredOutputRolePosture,
};
use crate::transactions::data::*;
use worth_execution::ExecutionRequest;

pub(super) fn refuse_late_claim() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (runtime, receipt) = super::super::incoming_reference::installed_references(1);
    let left = super::super::super::fixture::create(
        execution,
        &runtime,
        1,
        "outside-left",
        "authored left",
        None,
    );
    let right = super::super::super::fixture::create(
        execution,
        &runtime,
        1,
        "outside-right",
        "authored right",
        None,
    );
    let (outputs, edge, publisher) =
        produce_with_incidental(execution, &runtime, &receipt, left, right);
    let ProducerPublication {
        owner,
        installed,
        custody: birth,
    } = publisher;
    let birth_commit = birth.envelope.commit.clone();
    assert!(!birth
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .any(|row| row.id == edge));
    admit(execution, &runtime, &installed, outputs, false).unwrap();
    let left_payload = super::super::super::fixture::protected_payload(&runtime, left);
    let right_payload = super::super::super::fixture::protected_payload(&runtime, right);
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("native.closed.preserver", [20; 32]),
            birth.authority(),
        )
        .unwrap();
    let meaning = birth
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    let mut transaction = runtime
        .begin_branch_transaction(&basis(&runtime), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("real producer rewrites incidental reference").push(
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                    UpdateRelationEndpointsIntent {
                        relation_id: edge,
                        kind_id: KindId(4),
                        source: EntityReference::Existing(outputs[1]),
                        target: EntityReference::Existing(right),
                    },
                )),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: birth,
                meaning,
                roles: crate::mvcc::native_roles_fixture(
                    &outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Preserve,
                ),
            },
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the genuine incidental-reference Preserve must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let preserve_commit = committed.commit.clone();
    let preserved = committed.required_output_publication(&installed).unwrap();
    assert!(!preserved
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .any(|row| row.id == edge));
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    admit(execution, &runtime, &installed, outputs, true).unwrap();
    let saved = capture(&runtime);
    let decoded =
        crate::durability::log::native_file_codec::decode_checkpoint(execution, saved.bytes())
            .unwrap()
            .checkpoint;
    // The unmodified, genuinely independent reference remains admissible cold.
    let (cold, installed) = restore(execution, &saved);
    admit(execution, &cold, &installed, outputs, true).unwrap();
    assert_eq!(
        relation_payload(&cold, edge),
        relation_payload(&runtime, edge)
    );
    assert_eq!(
        super::super::super::fixture::protected_payload(&cold, left),
        left_payload
    );
    assert_eq!(
        super::super::super::fixture::protected_payload(&cold, right),
        right_payload
    );
    for forged_birth_endpoint in [false, true] {
        let mut forged = decoded.clone();
        // Clone an actual native row only to form hostile persisted claims; no custody is issued.
        let mut born = forged
            .envelopes
            .iter()
            .find(|entry| entry.commit.commit_id == birth_commit.commit_id)
            .unwrap()
            .required_output_origin
            .as_ref()
            .unwrap()
            .relations[0]
            .clone();
        born.id = edge;
        born.kind = KindId(4);
        born.source = if forged_birth_endpoint {
            outputs[0]
        } else {
            left
        };
        born.target = right;
        born.birth = birth_commit.commit_id;
        born.before_write = None;
        born.after_write = birth_commit.version_id;
        let mut continued = born.clone();
        continued.source = outputs[1];
        continued.before_write = Some(birth_commit.version_id);
        continued.after_write = preserve_commit.version_id;
        for (commit, claim) in [
            (birth_commit.commit_id, born),
            (preserve_commit.commit_id, continued),
        ] {
            edit_publication(&mut forged, commit, |envelope| {
                let mut origin: CanonicalRequiredOutputOrigin = rmp_serde::from_slice(
                    &rmp_serde::to_vec(envelope.required_output_origin.as_ref().unwrap().as_ref())
                        .unwrap(),
                )
                .unwrap();
                origin.relations.push(claim);
                origin.relations.sort_unstable_by_key(|row| row.id);
                envelope.required_output_origin = Some(std::sync::Arc::new(origin));
            });
        }
        let (bytes, _) = crate::durability::log::native_file_codec::encode_checkpoint(
            &forged,
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
        let (cold, installed) = restore(execution, &untrusted);
        let selected = basis(&cold);
        let before = capture(&cold);
        let denied = admit(execution, &cold, &installed, outputs, true).unwrap_err();
        assert!(
            format!("{denied:?}").contains(if forged_birth_endpoint {
                "relation birth roster differs from native allocation/endpoints"
            } else {
                "relation birth claim has no producer output endpoint"
            }),
            "{denied:?}"
        );
        assert!(selected.is_current());
        assert_eq!(capture(&cold), before);
        assert_eq!(
            relation_payload(&cold, edge),
            relation_payload(&runtime, edge)
        );
        assert_eq!(
            super::super::super::fixture::protected_payload(&cold, left),
            left_payload
        );
        assert_eq!(
            super::super::super::fixture::protected_payload(&cold, right),
            right_payload
        );
    }
}

fn restore(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
) -> (
    RelationalRuntime,
    crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
) {
    let (mut cold, receipt) = super::super::incoming_reference::installed_references(1);
    cold.durability_recovery()
        .restore_native_checkpoint(execution, checkpoint)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    (cold, installed)
}

fn admit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    outputs: [EntityId; 3],
    preserved: bool,
) -> Result<
    crate::mvcc::RelationalRequiredOutputPublication,
    crate::transactions::data::TransactionCommitError,
> {
    runtime.readmit_required_output_publication(
        execution,
        installed,
        &basis(runtime),
        if preserved {
            "native.closed.preserver"
        } else {
            "native.closed.producer"
        },
        if preserved { &[20; 32] } else { &[19; 32] },
        &crate::mvcc::native_meaning_fixture("native.closed.producer"),
        &crate::mvcc::native_roles_fixture(
            &outputs.map(|id| (id, KindId(3))),
            if preserved {
                RequiredOutputRolePosture::Preserve
            } else {
                RequiredOutputRolePosture::Create
            },
        ),
    )
}
